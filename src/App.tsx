import { Button } from "@/components/ui/button";

function App() {
  return (
    <main className="min-h-screen bg-background flex flex-col items-center justify-center">
      <div className="text-center space-y-6">
        <h1 className="text-5xl font-bold text-foreground">
          Mantra <span className="text-primary">心法</span>
        </h1>
        <p className="text-muted-foreground text-lg">
          AI 编程过程资产与方法论分享社区
        </p>
        <div className="flex gap-4 justify-center">
          <Button variant="default">开始探索</Button>
          <Button variant="outline">了解更多</Button>
        </div>
      </div>
    </main>
  );
}

export default App;
