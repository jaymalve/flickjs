import Navigation from '@/components/Navigation';
import Hero from '@/components/Hero';
import Tools from '@/components/Tools';
import FrameworkSpotlight from '@/components/FrameworkSpotlight';
import Installation from '@/components/Installation';
import Footer from '@/components/Footer';

const Index = () => {
  return (
    <div className="min-h-screen bg-background">
      <Navigation />
      <main>
        <Hero />
        <Tools />
        {/* <FrameworkSpotlight />
        <Installation /> */}
      </main>
      <Footer />
    </div>
  );
};

export default Index;
