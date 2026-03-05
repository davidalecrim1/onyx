import React from "react";

interface Props {
  children: React.ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends React.Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo): void {
    console.error("ErrorBoundary caught:", error, info.componentStack);
  }

  render(): React.ReactNode {
    if (this.state.hasError) {
      return (
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            justifyContent: "center",
            height: "100vh",
            backgroundColor: "#1a1d23",
            color: "#d4d4d8",
            fontFamily: "system-ui, sans-serif",
          }}
        >
          <h1 style={{ fontSize: "1.25rem", marginBottom: "1rem" }}>
            Something went wrong
          </h1>
          {this.state.error && (
            <pre
              style={{
                maxWidth: "80%",
                padding: "1rem",
                marginBottom: "1rem",
                backgroundColor: "#27272a",
                border: "1px solid #3f3f46",
                borderRadius: "0.375rem",
                fontSize: "0.75rem",
                color: "#ef4444",
                whiteSpace: "pre-wrap",
                wordBreak: "break-word",
                overflow: "auto",
                maxHeight: "40vh",
              }}
            >
              {this.state.error.message}
              {this.state.error.stack && `\n\n${this.state.error.stack}`}
            </pre>
          )}
          <button
            onClick={() => this.setState({ hasError: false, error: null })}
            style={{
              padding: "0.5rem 1rem",
              backgroundColor: "#3f3f46",
              color: "#d4d4d8",
              border: "1px solid #52525b",
              borderRadius: "0.375rem",
              cursor: "pointer",
              fontSize: "0.875rem",
            }}
          >
            Retry
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}
