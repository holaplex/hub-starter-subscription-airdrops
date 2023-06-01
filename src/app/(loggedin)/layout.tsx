'use client';
import { usePathname } from 'next/navigation';
import Hero from '../../components/Hero';
import Tabs from '../../layouts/Tabs';
import { cloneElement } from 'react';

export default async function Layout({
  children
}: {
  children: React.ReactNode;
}) {
  const pathname = usePathname();

  return (
    <>
      <Hero />

      <Tabs.Page className='mt-8'>
        <Tabs.Panel>
          <Tabs.Tab
            name='Past drips'
            href='/drips'
            active={pathname === '/drips'}
          />
          <Tabs.Tab
            name='Your collectibles'
            href='/collectibles'
            active={pathname === '/collectibles'}
          />
        </Tabs.Panel>
        <Tabs.Content>{cloneElement(children as JSX.Element)}</Tabs.Content>
      </Tabs.Page>
    </>
  );
}
