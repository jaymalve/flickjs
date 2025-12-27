// Enum for standardized framework names
export enum CompareFramework {
  SolidJS = "solid",
}

// Framework display config
export const frameworkConfig = {
  solid: {
    name: "SolidJS",
    color: "bg-blue-500",
    textColor: "text-blue-500",
    borderColor: "border-blue-500/30",
  },
} as const;
