import { Link } from 'react-router-dom';
import Navigation from '@/components/Navigation';
import Footer from '@/components/Footer';
import { ScanRulesContent } from '@/components/ScanRulesContent';

const ScanRules = () => {
  return (
    <div className="min-h-screen bg-background">
      <Navigation />
      <main>
        <section className="container py-12 lg:py-16">
          <div className="flex flex-col gap-4">
            <Link
              to="/scan"
              className="text-sm text-stone-500 hover:text-stone-300 transition-colors w-fit"
            >
              ← Flick Scan
            </Link>
            <h1 className="text-lg font-semibold tracking-tighter text-foreground">Rules</h1>
            <p className="text-base text-stone-400 leading-relaxed">
              Reference for every built-in lint rule and how it appears in config.
            </p>
          </div>
        </section>

        <section className="container pb-12 lg:pb-16">
          <div className="flex flex-col gap-3">
            <ScanRulesContent />
          </div>
        </section>
      </main>
      <Footer />
    </div>
  );
};

export default ScanRules;
