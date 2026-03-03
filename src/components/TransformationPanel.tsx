import { useState } from "react";
import RenameTab from "./RenameTab";
import ConvertTab from "./ConvertTab";
import MetadataTab from "./MetadataTab";
import type { RenamePattern } from "@/types";

export type TabType = "rename" | "convert" | "metadata";

interface TransformationPanelProps {
  isOpen: boolean;
  onToggle: () => void;
  onPatternChange?: (pattern: RenamePattern | null) => void;
}

export default function TransformationPanel({
  isOpen,
  onToggle,
  onPatternChange,
}: TransformationPanelProps) {
  const [activeTab, setActiveTab] = useState<TabType>("rename");

  if (!isOpen) {
    return (
      <button
        onClick={onToggle}
        className="absolute right-0 top-1/2 z-10 -translate-y-1/2 rounded-l-lg border border-r-0 border-border bg-bg-card p-2 text-text-secondary transition-colors duration-200 hover:bg-bg-card-hover hover:text-text-primary"
        aria-label="Open transformation panel"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
          aria-hidden="true"
        >
          <polyline points="15 18 9 12 15 6" />
        </svg>
      </button>
    );
  }

  const tabs: { id: TabType; label: string }[] = [
    { id: "rename", label: "Rename" },
    { id: "convert", label: "Convert" },
    { id: "metadata", label: "Metadata" },
  ];

  return (
    <aside className="flex w-80 shrink-0 flex-col border-l border-border bg-bg-card">
      {/* Tab header */}
      <div className="flex items-center justify-between border-b border-border px-2 py-1">
        <div className="flex" role="tablist" aria-label="Transformation options">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              role="tab"
              aria-selected={activeTab === tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`rounded-md px-3 py-1.5 text-sm font-medium transition-colors duration-200 ${
                activeTab === tab.id
                  ? "bg-accent/10 text-accent"
                  : "text-text-secondary hover:text-text-primary"
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>
        <button
          onClick={onToggle}
          className="rounded-md p-1 text-text-secondary transition-colors duration-200 hover:bg-bg-card-hover hover:text-text-primary"
          aria-label="Close transformation panel"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
            aria-hidden="true"
          >
            <polyline points="9 18 15 12 9 6" />
          </svg>
        </button>
      </div>

      {/* Tab content */}
      <div className="flex-1 overflow-y-auto p-4" role="tabpanel">
        {activeTab === "rename" && <RenameTab onPatternChange={onPatternChange} />}
        {activeTab === "convert" && <ConvertTab />}
        {activeTab === "metadata" && <MetadataTab />}
      </div>
    </aside>
  );
}
