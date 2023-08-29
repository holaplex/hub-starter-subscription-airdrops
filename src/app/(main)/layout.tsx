import Airdrops from './Airdrops';

export default async function HomeLayout({
  children
}: {
  children: React.ReactNode;
}) {
  return <Airdrops>{children}</Airdrops>;
}
