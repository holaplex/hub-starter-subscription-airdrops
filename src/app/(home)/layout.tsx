import Home from './Home';

export default async function HomeLayout({
  children
}: {
  children: React.ReactNode;
}) {
  return <Home>{children}</Home>;
}
