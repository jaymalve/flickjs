"use client";

// Core reactive primitives (pure JS, work anywhere)
export { fx, run } from "@flickjs/runtime";
export type { Fx } from "@flickjs/runtime";

// React hooks
export { useFx } from "./hooks/use-fx";
export { useComputed } from "./hooks/use-computed";
export { useFxEffect } from "./hooks/use-fx-effect";
