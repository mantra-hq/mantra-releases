import * as React from "react";
import { ThemeToggle } from "@/components/theme-toggle";
import { DualStreamLayout, type DualStreamLayoutRef } from "@/components/layout";
import { MOCK_MESSAGES } from "@/lib/mock-messages";
import type { NarrativeMessage } from "@/types/message";

function App() {
  // DualStreamLayout ref 用于程序化滚动
  const layoutRef = React.useRef<DualStreamLayoutRef>(null);

  // 选中的消息 ID
  const [selectedMessageId, setSelectedMessageId] = React.useState<string | undefined>();

  // 消息选中回调
  const handleMessageSelect = React.useCallback(
    (messageId: string, message: NarrativeMessage) => {
      setSelectedMessageId(messageId);
      console.log("Selected message:", message);
    },
    []
  );

  // 滚动到指定消息的测试函数 (可在 DevTools 中调用，仅开发环境)
  React.useEffect(() => {
    if (import.meta.env.DEV) {
      // 暴露到 window 供调试
      (window as unknown as { scrollToMessage: (id: string) => void }).scrollToMessage = (id: string) => {
        layoutRef.current?.scrollToMessage(id);
      };
    }
  }, []);

  return (
    <div className="h-screen flex flex-col bg-background">
      {/* Header with Theme Toggle */}
      <header className="shrink-0 sticky top-0 z-50 w-full border-b border-border bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
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

      {/* Main Content - DualStreamLayout */}
      <main className="flex-1 min-h-0 overflow-hidden">
        <DualStreamLayout
          ref={layoutRef}
          messages={MOCK_MESSAGES}
          selectedMessageId={selectedMessageId}
          onMessageSelect={handleMessageSelect}
        />
      </main>
    </div>
  );
}

export default App;
