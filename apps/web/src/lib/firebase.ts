import { initializeApp, getApps, type FirebaseApp } from 'firebase/app';
import { getAuth, GoogleAuthProvider, GithubAuthProvider, type Auth } from 'firebase/auth';

// Public Firebase config — these are safe to expose in client code.
// Firebase security is enforced by Auth rules, not by hiding the config.
const firebaseConfig = {
  apiKey:            process.env.NEXT_PUBLIC_FIREBASE_API_KEY            ?? '',
  authDomain:        process.env.NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN        ?? '',
  projectId:         process.env.NEXT_PUBLIC_FIREBASE_PROJECT_ID         ?? '',
  storageBucket:     process.env.NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET     ?? '',
  messagingSenderId: process.env.NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID ?? '',
  appId:             process.env.NEXT_PUBLIC_FIREBASE_APP_ID             ?? '',
  measurementId:     process.env.NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID     ?? '',
};

// True when all required env vars are present — used to conditionally
// render the real OAuth buttons vs the disabled placeholder buttons.
export const firebaseConfigured = Boolean(
  firebaseConfig.apiKey &&
  firebaseConfig.authDomain &&
  firebaseConfig.projectId,
);

// Only initialise Firebase when fully configured to avoid throwing during
// SSR / build-time pre-rendering when env vars are not available.
let app: FirebaseApp | null = null;
let _auth: Auth | null = null;

if (firebaseConfigured) {
  app = getApps().length === 0 ? initializeApp(firebaseConfig) : getApps()[0];
  _auth = getAuth(app);
}

// Cast is safe: callers always check `firebaseConfigured` before using auth.
export const auth = _auth as Auth;

export const googleProvider = new GoogleAuthProvider();
googleProvider.addScope('email');
googleProvider.addScope('profile');

export const githubProvider = new GithubAuthProvider();
githubProvider.addScope('user:email');
