const ZAT_PER_ZEC = 100_000_000;

/** Splits a zatoshi amount into a whole part and up to 8 decimals, trailing zeros trimmed. */
export function zec(zatoshis: number): { whole: string; fraction: string } {
  const whole = Math.floor(zatoshis / ZAT_PER_ZEC);
  const fraction = String(zatoshis % ZAT_PER_ZEC)
    .padStart(8, "0")
    .replace(/0+$/, "");
  return { whole: whole.toLocaleString("en-US"), fraction };
}

export function zecText(zatoshis: number): string {
  const { whole, fraction } = zec(zatoshis);
  return fraction ? `${whole}.${fraction}` : whole;
}

export const blockHeight = (h: number) => h.toLocaleString("en-US");

/** ZIP 318 leaves Orchard value below 0.01 ZEC behind as unmigratable dust. */
export const DUST_ZATOSHIS = 1_000_000;
