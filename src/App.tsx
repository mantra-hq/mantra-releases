import { Button } from "@/components/ui/button";
import { ThemeToggle } from "@/components/theme-toggle";

function App() {
  return (
    <div className="min-h-screen bg-background">
      {/* Header with Theme Toggle */}
      <header className="sticky top-0 z-50 w-full border-b border-border bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="flex h-14 items-center justify-between px-4">
          <div className="flex items-center gap-2">
            <span className="text-xl font-bold text-foreground">
              Mantra <span className="text-primary">心法</span>
            </span>
          </div>
          <div className="flex items-center gap-2">
            <ThemeToggle />
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="flex flex-col items-center justify-center py-24">
        <div className="text-center space-y-6">
          <h1 className="text-5xl font-bold text-foreground">
            Mantra <span className="text-primary">心法</span>
          </h1>
          <p className="text-muted-foreground text-lg max-w-md">
            AI 编程过程资产与方法论分享社区
          </p>
          <div className="flex gap-4 justify-center">
            <Button variant="default">开始探索</Button>
            <Button variant="outline">了解更多</Button>
          </div>
        </div>
      </main>
    </div>
  );
}

export default App;
