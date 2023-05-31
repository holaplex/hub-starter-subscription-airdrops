import { getServerSession } from 'next-auth/next';
import { authOptions } from '@/pages/api/auth/[...nextauth]';
import { redirect } from 'next/navigation';
import Login from './Login';

export default async function LoginPage() {
  const session = await getServerSession(authOptions);
  console.log('session', session);
  if (session) {
    redirect('/');
  }

  return <Login />;
}
