import { LoginForm } from '@/shared/ui/organisms';
import { AuthTemplate } from '@/shared/ui/templates';

/**
 * Login page composing auth template and login form.
 *
 * @returns Login screen.
 */
export function LoginPage() {
  return (
    <AuthTemplate>
      <LoginForm />
    </AuthTemplate>
  );
}
