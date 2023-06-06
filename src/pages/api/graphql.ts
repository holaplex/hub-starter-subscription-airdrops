import { ApolloServer } from '@apollo/server';
import { ApolloClient, NormalizedCacheObject } from '@apollo/client';
import { startServerAndCreateNextHandler } from '@as-integrations/next';
import db from '@/modules/db';
import {
  MintDropInput,
  MintEditionPayload,
  MutationResolvers,
  QueryResolvers,
  Project,
  CollectionMint,
  Drop,
  Subscription
} from '@/graphql.types';
import { Session } from 'next-auth';
import { MintNft } from '@/mutations/drop.graphql';
import { PrismaClient } from '@prisma/client';
import { getServerSession } from 'next-auth/next';
import { GetProjectDrop, GetProjectDrops } from '@/queries/project.graphql';
import { GetCustomerCollectibles } from '@/queries/customer.graphql';
import { authOptions } from '@/pages/api/auth/[...nextauth]';
import UserSource from '@/modules/user';
import holaplex from '@/modules/holaplex';
import { loadSchema } from '@graphql-tools/load';
import { GraphQLFileLoader } from '@graphql-tools/graphql-file-loader';
import { subscribe } from 'graphql';

export interface AppContext {
  session: Session | null;
  dataSources: {
    db: PrismaClient;
    holaplex: ApolloClient<NormalizedCacheObject>;
    user: UserSource;
  };
}

interface GetDropVars {
  project: string;
  drop: string;
}

interface GetDropData {
  project: Pick<Project, 'drop'>;
}

interface GetDropVars {
  project: string;
  drop: string;
}

interface GetDropsData {
  project: Pick<Project, 'drops'>;
}

interface GetDropsVars {
  project: string;
}

interface GetCustomerCollectiblesData {
  project: Pick<Project, 'customer'>;
}

interface GetCustomerCollectiblesVars {
  project: string;
  customer: string;
}

export const queryResolvers: QueryResolvers<AppContext> = {
  async drop(_a, _b, { dataSources: { holaplex } }) {
    const { data } = await holaplex.query<GetDropData, GetDropVars>({
      fetchPolicy: 'network-only',
      query: GetProjectDrop,
      variables: {
        project: process.env.HOLAPLEX_PROJECT_ID as string,
        drop: process.env.HOLAPLEX_DROP_ID as string
      }
    });

    return data.project.drop as Drop;
  },
  async drops(_a, _b, { dataSources: { holaplex } }) {
    const { data } = await holaplex.query<GetDropsData, GetDropsVars>({
      fetchPolicy: 'network-only',
      query: GetProjectDrops,
      variables: {
        project: process.env.HOLAPLEX_PROJECT_ID as string
      }
    });

    return data.project.drops as [Drop];
  },
  async collectibles(_a, _b, { session, dataSources: { holaplex, db } }) {
    console.log('Get collectibles');
    if (!session) {
      return null;
    }
    const user = await db.user.findFirst({
      where: { email: session.user?.email }
    });

    console.log('user', user);

    if (!user || !user.holaplexCustomerId) {
      return null;
    }

    const { data } = await holaplex.query<
      GetCustomerCollectiblesData,
      GetCustomerCollectiblesVars
    >({
      fetchPolicy: 'network-only',
      query: GetCustomerCollectibles,
      variables: {
        project: process.env.HOLAPLEX_PROJECT_ID as string,
        customer: user?.holaplexCustomerId
      }
    });

    return data.project.customer?.mints as [CollectionMint];
  },
  async subscription(_a, _b, { session, dataSources: { db } }) {
    if (!session) {
      return null;
    }
    const user = await db.user.findFirst({
      where: { email: session.user?.email }
    });

    if (!user || !user.holaplexCustomerId) {
      return null;
    }

    const subscription = await db.subscription.findFirst({
      where: { userId: user.id }
    });

    return {
      userId: user.id,
      subscribedAt: subscription?.subscribedAt
    } as Subscription;
  },
  async me(_a, _b, { session, dataSources: { user } }) {
    if (!session) {
      return null;
    }

    const me = await user.get(session.user?.email);

    if (me) {
      return me;
    }

    return null;
  }
};

interface MintNftData {
  mintEdition: MintEditionPayload;
}

interface MintNftVars {
  input: MintDropInput;
}

const mutationResolvers: MutationResolvers<AppContext> = {
  async subscribe(_a, _b, { session, dataSources: { db, holaplex } }) {
    if (!session) {
      return null;
    }

    const user = await db.user.findFirst({
      where: { email: session.user?.email }
    });

    const subscription = await db.subscription.create({
      data: {
        userId: user?.id as string,
        subscribedAt: new Date()
      }
    });

    return {
      userId: user?.id,
      subscribedAt: subscription.subscribedAt?.toISOString()
    } as Subscription;
  },
  async unsubscribe(_a, _b, { session, dataSources: { db, holaplex } }) {
    if (!session) {
      return null;
    }

    const user = await db.user.findFirst({
      where: { email: session.user?.email }
    });

    const subscription = await db.subscription.update({
      where: {
        userId: user?.id as string
      },
      data: {
        subscribedAt: null
      }
    });
    return {
      userId: user?.id,
      subscribedAt: subscription.subscribedAt?.toISOString()
    } as Subscription;
  },
  async mint(_a, _b, { session, dataSources: { db, holaplex } }) {
    if (!session) {
      return null;
    }

    const wallet = await db.wallet.findFirst({
      where: {
        user: {
          email: session?.user?.email
        }
      }
    });

    const { data } = await holaplex.mutate<MintNftData, MintNftVars>({
      mutation: MintNft,
      variables: {
        input: {
          drop: process.env.HOLAPLEX_DROP_ID as string,
          recipient: wallet?.address as string
        }
      }
    });

    return data?.mintEdition.collectionMint as CollectionMint;
  }
};

const typeDefs = await loadSchema('./schema.graphql', {
  loaders: [new GraphQLFileLoader()]
});

const server = new ApolloServer<AppContext>({
  resolvers: {
    Query: queryResolvers,
    Mutation: mutationResolvers
  },
  typeDefs
});

export default startServerAndCreateNextHandler(server, {
  context: async (req, res) => {
    const session = await getServerSession(req, res, authOptions);

    return {
      session,
      dataSources: {
        db,
        holaplex,
        user: new UserSource(holaplex, db)
      }
    };
  }
});
