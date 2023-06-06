import { Drop } from '../src/graphql.types';
import holaplex from '../src/modules/holaplex';
import { GetProjectDrops } from '../src/queries/project.graphql';

interface GetDropsData {
  drops: [Drop];
}

interface GetDropsVars {
  project: string;
}

export const init = async () => {
  console.log('Project id', process.env.HOLAPLEX_PROJECT_ID);
  const { data } = await holaplex.query<GetDropsData, GetDropsVars>({
    fetchPolicy: 'network-only',
    query: GetProjectDrops,
    variables: {
      project: process.env.HOLAPLEX_PROJECT_ID as string
    }
  });

  data.drops.map((drop: Drop) => {
    console.log(drop);
  });
};

(async () => {
  await init();
})();
