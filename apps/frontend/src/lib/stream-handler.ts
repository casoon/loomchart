/**
 * Stream Handler
 *
 * Manages real-time candle updates with proper time rounding,
 * candle detection, and countdown timer for next candle.
 */

export interface Candle {
  time: number;
  o: number;
  h: number;
  l: number;
  c: number;
  v: number;
}

const SECOND_MS = 1000;
const MINUTE_MS = 60 * SECOND_MS;
const HOUR_MS = 60 * MINUTE_MS;

/**
 * Parse timeframe string to milliseconds
 */
export function timeframeToMs(timeframe: string): number {
  const match = timeframe.match(/^(\d+)([smhd])$/);
  if (!match) {
    console.warn(`[StreamHandler] Invalid timeframe: ${timeframe}, defaulting to 1h`);
    return HOUR_MS;
  }

  const value = parseInt(match[1], 10);
  const unit = match[2];

  switch (unit) {
    case "s":
      return value * SECOND_MS;
    case "m":
      return value * MINUTE_MS;
    case "h":
      return value * HOUR_MS;
    case "d":
      return value * 24 * HOUR_MS;
    default:
      return HOUR_MS;
  }
}

export class StreamHandler {
  private timeframeMs: number;
  private countdownInterval: number | null = null;
  private currentCandle: Candle | null = null;
  private lastUpdate: number = 0;
  private maxUpdateInterval: number = 100; // Max 100ms between updates

  private onUpdateCallback: (candle: Candle, isNew: boolean) => void;
  private onCountdownCallback: (remaining: string) => void;

  constructor(
    timeframe: string,
    onUpdate: (candle: Candle, isNew: boolean) => void,
    onCountdown: (remaining: string) => void,
  ) {
    this.timeframeMs = timeframeToMs(timeframe);
    this.onUpdateCallback = onUpdate;
    this.onCountdownCallback = onCountdown;

    console.log("[StreamHandler] Initialized with timeframe:", timeframe, "=", this.timeframeMs, "ms");
  }

  /**
   * Process incoming candle data
   */
  processCandle(data: Candle): void {
    const roundedTime = this.roundTime(data.time);
    const candle: Candle = { ...data, time: roundedTime };

    const isNewCandle =
      !this.currentCandle || this.currentCandle.time !== roundedTime;

    if (isNewCandle) {
      console.log("[StreamHandler] New candle detected:", new Date(roundedTime * 1000).toISOString());
      this.currentCandle = candle;
    } else {
      // Update existing candle
      this.currentCandle = {
        time: roundedTime,
        o: this.currentCandle.o, // Keep original open
        h: Math.max(this.currentCandle.h, candle.h),
        l: Math.min(this.currentCandle.l, candle.l),
        c: candle.c,
        v: candle.v, // Use latest volume
      };
    }

    // Throttle updates
    const now = Date.now();
    if (now - this.lastUpdate >= this.maxUpdateInterval) {
      this.onUpdateCallback(this.currentCandle, isNewCandle);
      this.lastUpdate = now;
    }
  }

  /**
   * Round timestamp to timeframe boundary
   */
  roundTime(timestamp: number): number {
    // Convert to milliseconds if it's in seconds
    const ts = timestamp < 10000000000 ? timestamp * 1000 : timestamp;
    const rounded = ts - (ts % this.timeframeMs);
    // Return in seconds
    return Math.floor(rounded / 1000);
  }

  /**
   * Start countdown timer
   */
  startCountdown(): void {
    if (this.countdownInterval) {
      clearInterval(this.countdownInterval);
    }

    this.updateCountdown(); // Initial update

    this.countdownInterval = window.setInterval(() => {
      this.updateCountdown();
    }, 1000); // Update every second

    console.log("[StreamHandler] Countdown started");
  }

  /**
   * Stop countdown timer
   */
  stopCountdown(): void {
    if (this.countdownInterval) {
      clearInterval(this.countdownInterval);
      this.countdownInterval = null;
      console.log("[StreamHandler] Countdown stopped");
    }
  }

  /**
   * Update countdown display
   */
  private updateCountdown(): void {
    if (!this.currentCandle) {
      this.onCountdownCallback("--:--");
      return;
    }

    const now = Date.now();
    const candleStartMs = this.currentCandle.time * 1000;
    const candleEndMs = candleStartMs + this.timeframeMs;
    const remainingMs = candleEndMs - now;

    if (remainingMs <= 0) {
      this.onCountdownCallback("00:00");
      return;
    }

    const formatted = this.formatCountdown(remainingMs);
    this.onCountdownCallback(formatted);
  }

  /**
   * Format countdown time
   */
  private formatCountdown(ms: number): string {
    const totalSeconds = Math.floor(ms / SECOND_MS);

    if (ms >= HOUR_MS) {
      // HH:MM:SS format
      const h = Math.floor(ms / HOUR_MS);
      const m = Math.floor((ms % HOUR_MS) / MINUTE_MS);
      const s = Math.floor((ms % MINUTE_MS) / SECOND_MS);
      return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
    } else if (ms >= MINUTE_MS) {
      // MM:SS format
      const m = Math.floor(ms / MINUTE_MS);
      const s = Math.floor((ms % MINUTE_MS) / SECOND_MS);
      return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
    } else {
      // SS format
      return `${totalSeconds}s`;
    }
  }

  /**
   * Update timeframe
   */
  setTimeframe(timeframe: string): void {
    this.timeframeMs = timeframeToMs(timeframe);
    this.currentCandle = null;
    console.log("[StreamHandler] Timeframe updated to:", timeframe, "=", this.timeframeMs, "ms");
  }

  /**
   * Cleanup
   */
  destroy(): void {
    this.stopCountdown();
    this.currentCandle = null;
    console.log("[StreamHandler] Destroyed");
  }
}
