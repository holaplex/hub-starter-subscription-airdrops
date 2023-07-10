'use client';
import { GetPastDrips } from '@/queries/airdrops.graphql';
import { useQuery } from '@apollo/client';
import { Airdrop } from '@/graphql.types';

interface GetPastDripsData {
  pastDrips: [Airdrop];
}

export default async function DripsPage() {
  const pastDripsQuery = useQuery<GetPastDripsData>(GetPastDrips);

  return (
    <div>
      <div className='flex flex-wrap gap-6 justify-center mt-4 mb-10'>
        {pastDripsQuery.loading ? (
          <>
            {Array.from(Array(8)).map((_, index) => (
              <div key={index}>
                <div className='w-52 h-52 rounded-lg bg-gray-600 animate-pulse' />
              </div>
            ))}
          </>
        ) : (
          <>
            {pastDripsQuery.data?.pastDrips?.map((airdrop: Airdrop) => {
              return (
                <div key={airdrop.drop?.id}>
                  <img
                    className='w-52 h-52 rounded-lg object-contain'
                    src={airdrop.drop?.collection.metadataJson?.image as string}
                  />
                </div>
              );
            })}
          </>
        )}
      </div>
    </div>
  );
}
