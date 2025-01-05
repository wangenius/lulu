import { useRef, useEffect } from "react";
import { Message } from "../types";
import { CopyIcon } from "lucide-react";

interface ChatViewProps {
  messages: Message[];
}

export function ChatView({ messages }: ChatViewProps) {
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const lastAssistantMessage = useRef<string>("");

  useEffect(() => {
    if (messagesEndRef.current) {
      messagesEndRef.current.scrollIntoView({ behavior: "smooth" });
    }
    const assistantMessages = messages.filter(m => m.bot === "assistant");
    if (assistantMessages.length > 0) {
      lastAssistantMessage.current = assistantMessages[assistantMessages.length - 1].content;
    }
  }, [messages]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key === 'c') {
        const selection = window.getSelection();
        if (!selection || selection.rangeCount === 0 || selection.toString().trim() === '') {
          e.preventDefault();
          navigator.clipboard.writeText(lastAssistantMessage.current);
        }
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, []);

  const copyMessage = (content: string) => {
    navigator.clipboard.writeText(content);
  };

  return (
    <div className="max-w-4xl mx-auto p-2">
      {messages.map((message, index) => (
        <div key={index} className="group">
          <div className={`
            py-2 px-4 mt-2 rounded-xl
            ${message.bot === "assistant" ? "bg-slate-50 w-fit" : "bg-white"}
          `}>
            <div className="relative">
              <div className="text-sm leading-relaxed whitespace-pre-wrap text-gray-800">
                {message.content}
              </div>
            </div>
          </div>
        </div>
      ))}
      <div ref={messagesEndRef} />
    </div>
  );
} 