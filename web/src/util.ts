// Small helpers shared by the game and the character creator.

/** The first element matching `selector`; callers only query markup they built. */
export const $ = <T extends HTMLElement = HTMLElement>(selector: string) => document.querySelector(selector) as T;

export const randomSeed = () => Math.floor(Math.random() * 1e9);

/** Parses a JSON string returned by the WASM engine into its known shape. */
export const parseJson = <T>(json: string) => JSON.parse(json) as T;
