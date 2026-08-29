export function writtenLiterally(): void {}

// A declaration that exists only at runtime.
(globalThis as Record<string, unknown>)["generated"] = writtenLiterally;
