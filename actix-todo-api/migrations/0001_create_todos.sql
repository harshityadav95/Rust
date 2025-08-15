-- Create todos table
CREATE TABLE IF NOT EXISTS todos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    completed INTEGER NOT NULL DEFAULT 0, -- 0=false, 1=true
    due_date TEXT,                        -- RFC3339 text
    created_at TEXT NOT NULL,             -- RFC3339 text
    updated_at TEXT NOT NULL              -- RFC3339 text
);

-- Helpful indexes
CREATE INDEX IF NOT EXISTS idx_todos_completed ON todos(completed);
CREATE INDEX IF NOT EXISTS idx_todos_due_date ON todos(due_date);
CREATE INDEX IF NOT EXISTS idx_todos_created_at ON todos(created_at);