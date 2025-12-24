import PlaceholderPage from '../components/common/PlaceholderPage';

export default function GoalsPage() {
  return (
    <PlaceholderPage
      titleKey="nav.goals"
      icon={
        <svg className="w-10 h-10" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M13 10V3L4 14h7v7l9-11h-7z" />
        </svg>
      }
    />
  );
}
