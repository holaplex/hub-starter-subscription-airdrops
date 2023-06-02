import Drips from './Drips';

export default async function HomeLayout({
  children
}: {
  children: React.ReactNode;
}) {
  return <Drips>{children}</Drips>;
}
