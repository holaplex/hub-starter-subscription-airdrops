'use client';
import Image from 'next/image';
import Link from 'next/link';
import useMe from '@/hooks/useMe';
import { PopoverBox } from './Popover';
import { signOut } from 'next-auth/react';
import { shorten } from '../modules/wallet';
import { Icon } from './Icon';
import Copy from './Copy';

export default function Hero() {
  const me = useMe();
  console.log('me', me);
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
          <PopoverBox
            triggerButton={
              <button className='text-cta font-bold border-2 rounded-full border-cta py-3 px-6 flex gap-2 items-center'>
                <img
                  className='w-6 h-6 rounded-full'
                  src={me?.image as string}
                />
                <span>{me?.name}</span>
                <Icon.ChevronDown className='stroke-cta' />
              </button>
            }
          >
            <div className='rounded-lg bg-gray-200 p-6 flex flex-col items-center mt-4'>
              <span className='text-xs text-gray-300'>
                Solana wallet address
              </span>
              <div className='flex gap-2 mt-1'>
                <span className='text-xs'>
                  {shorten(me.wallet?.address as string)}
                </span>
                <Copy copyString={me.wallet?.address as string} />
              </div>
              <button
                onClick={() => signOut()}
                className='text-cta font-medium md:font-bold md:border-2 md:rounded-full md:border-cta md:py-3 md:px-6 mt-10'
              >
                Log out
              </button>
            </div>
          </PopoverBox>
        )}
      </div>
      <div className='w-full flex flex-col items-center'>
        <div className='flex flex-col items-center mt-20'>
          <span className='font-semibold text-5xl'>Holaplex drip</span>

          <span className='font-medium mt-3 text-gray-500'>
            Sign up to receive new collectibles each week!
          </span>

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
