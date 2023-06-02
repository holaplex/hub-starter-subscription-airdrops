'use client';
import { GetDrops } from '@/queries/drop.graphql';
import { useQuery } from '@apollo/client';
import { Drop } from '../../graphql.types';

interface GetDropsData {
  drops: [Drop];
}

export default function DripsPage() {
  const dropsQuery = useQuery<GetDropsData>(GetDrops);

  return (
    <div>
      <div className='flex flex-wrap gap-6 justify-center mt-4 mb-10'>
        {dropsQuery.loading ? (
          <>
            {Array.from(Array(8)).map((index) => (
              <div key={index}>
                <div className='w-52 h-52 rounded-lg bg-gray-600 animate-pulse' />
              </div>
            ))}
          </>
        ) : (
          <>
            {dropsQuery.data?.drops.map((drop: Drop) => (
              <div key={drop.id}>
                <img
                  className='w-52 h-52 rounded-lg object-contain'
                  src={drop.collection.metadataJson?.image as string}
                />
              </div>
            ))}
          </>
        )}
      </div>
    </div>
  );
}
