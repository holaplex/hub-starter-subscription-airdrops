'use client';
import { GetPastAirdrops } from '@/queries/airdrops.graphql';
import { useQuery } from '@apollo/client';
import { Airdrop } from '@/graphql.types';

interface GetPastAirdropsData {
  pastAirdrops: [Airdrop];
}

export default function AirdropsPage() {
  const pastAirdropsQuery = useQuery<GetPastAirdropsData>(GetPastAirdrops);

  return (
    <div>
      <div className='flex flex-wrap gap-6 justify-center mt-4 mb-10'>
        {pastAirdropsQuery.loading ? (
          <>
            {Array.from(Array(8)).map((_, index) => (
              <div key={index}>
                <div className='w-52 h-52 rounded-lg bg-gray-600 animate-pulse' />
              </div>
            ))}
          </>
        ) : (
          <>
            {pastAirdropsQuery.data?.pastAirdrops?.map((airdrop: Airdrop) => {
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
