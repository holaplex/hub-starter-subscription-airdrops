'use client';
import Image from 'next/image';
import { cloneElement, useMemo } from 'react';
import { Holder } from '@/graphql.types';
import { shorten } from '../modules/wallet';
import { MintDrop } from '@/mutations/mint.graphql';
import { useApolloClient, useMutation, useQuery } from '@apollo/client';
import { GetDrop } from '@/queries/drop.graphql';
import BounceLoader from 'react-spinners/BounceLoader';
import Link from 'next/link';
import clsx from 'clsx';
import { drop, isNil, not, pipe } from 'ramda';
import useMe from '@/hooks/useMe';
import { Session } from 'next-auth';
import { CheckIcon } from '@heroicons/react/24/solid';
import Tabs from '../layouts/Tabs';
import { usePathname } from 'next/navigation';

interface MintData {
  mint: string;
}

export default function Hero() {
  const me = useMe();
  const dropQuery = useQuery(GetDrop);
  const pathname = usePathname();
  const collection = dropQuery.data?.drop.collection;
  const metadataJson = collection?.metadataJson;
  const holder = useMemo(() => {
    return collection?.holders?.find(
      (holder: Holder) => holder.address === me?.wallet?.address
    );
  }, [collection?.holders, me?.wallet]);
  const owns = pipe(isNil, not)(holder);
  const [mint, { loading }] = useMutation<MintData>(MintDrop, {
    awaitRefetchQueries: true,
    refetchQueries: [
      {
        query: GetDrop
      }
    ]
  });

  const onMint = () => {
    mint();
  };

  return (
    <>
      <div className='flex w-full justify-between items-center py-4'>
        <Image src='/img/logo.png' alt='site logo' width={199} height={18} />
        {!me ? (
          <>
            <div className='flex gap-1 md:gap-4 items-center'>
              <Link
                href='/login'
                className='text-cta font-medium md:font-bold md:border-2 md:rounded-full md:border-cta md:py-3 md:px-6'
              >
                Log in
              </Link>
              <span className='text-gray-300 font-medium md:hidden'>or</span>
              <Link
                href='/login'
                className='text-cta font-medium md:text-backdrop md:bg-cta md:rounded-full md:font-bold md:py-3 md:px-6'
              >
                Sign up
              </Link>
            </div>
          </>
        ) : (
          <button className='text-cta font-bold border-2 rounded-full border-cta py-3 px-6 flex gap-2'>
            <img className='w-6 h-6 rounded-full' src={me?.image as string} />
            <span>{me?.name}</span>
          </button>
        )}
      </div>
      <div className='w-full flex flex-col items-center'>
        <div className='flex flex-col items-center mt-20'>
          <span className='font-semibold text-5xl'>Holaplex drip</span>
          {!me && (
            <span className='font-medium mt-3 text-gray-500'>
              Sign up to receive new collectibles each week!
            </span>
          )}

          <Link
            href='/'
            className='text-cta font-bold md:text-backdrop md:bg-cta md:rounded-full md:font-bold md:py-3 md:px-6 mt-10'
          >
            Subscribe for free
          </Link>
        </div>
      </div>
    </>
  );
}
