import { gql } from '@apollo/client';
import { Drop, Project } from '../src/graphql.types';
import holaplex from '../src/modules/holaplex';
import db from '../src/modules/db';
import { Subscription } from '@prisma/client';
import { MintEditionPayload } from '../src/graphql.types';
import { MintDropInput } from '../src/graphql.types';

interface MintNftData {
  mintEdition: MintEditionPayload;
}

interface MintNftVars {
  input: MintDropInput;
}

interface GetDropsData {
  project: Project;
}

interface GetDropsVars {
  project: string;
}

const MINT_MUTATION = gql`
  mutation ($input: MintDropInput!) {
    mintEdition(input: $input) {
      collectionMint {
        address
        owner
      }
    }
  }
`;

const DROPS_QUERY = gql`
  query ($project: UUID!) {
    project(id: $project) {
      id
      drops {
        id
        startTime
        endTime
        collection {
          totalMints
          supply
          id
          address
          holders {
            address
            owns
          }
          metadataJson {
            id
            image
            name
            description
            attributes {
              traitType
              value
            }
          }
        }
      }
    }
  }
`;

export const start = async () => {
  console.log('Start Airdrop');
  console.log('Project id', process.env.HOLAPLEX_PROJECT_ID);
  const result = await holaplex.query<GetDropsData, GetDropsVars>({
    fetchPolicy: 'network-only',
    query: DROPS_QUERY,
    variables: {
      project: process.env.HOLAPLEX_PROJECT_ID as string
    }
  });

  result.data.project.drops?.map(async (drop: Drop) => {
    console.log(drop);
    const airdrop = await db.airdrop.findFirst({
      where: { dropId: drop.id }
    });

    console.log('airdrop', airdrop);

    if (!drop.startTime || new Date(drop.startTime) <= new Date())
      console.log('Drop open for minting', drop.id);
    if (!airdrop || !airdrop.completedAt) {
      // Set Airdrop starttime
      const updateStartTime = await db.airdrop.upsert({
        where: { dropId: drop.id },
        update: { startedAt: new Date() },
        create: { dropId: drop.id, startedAt: new Date() }
      });
      console.log('Add starttime', updateStartTime);

      // Airdrop
      console.log('Start minting');
      await mint(drop.id);

      // Set Airdrop endtime
      const updateEndTime = await db.airdrop.upsert({
        where: { dropId: drop.id },
        update: { dropId: drop.id, completedAt: new Date() },
        create: { dropId: drop.id, completedAt: new Date() }
      });
      console.log('Add endTime', updateEndTime);
    }
  });
};

const mint = async (dropId: string) => {
  const subscriptions = await db.subscription.findMany({
    where: {
      subscribedAt: {
        not: null
      }
    }
  });

  console.log('Subscriptions', subscriptions);
  subscriptions.map(async (sub: Subscription) => {
    const wallet = await db.wallet.findFirst({
      where: {
        user: {
          id: sub.userId
        }
      }
    });
    console.log('Wallet', wallet);

    const result = await holaplex.mutate<MintNftData, MintNftVars>({
      mutation: MINT_MUTATION,
      variables: {
        input: {
          drop: dropId,
          recipient: wallet?.address as string
        }
      }
    });

    console.log('Mint result', result.data?.mintEdition);
  });
};

(async () => {
  await start();
})();
