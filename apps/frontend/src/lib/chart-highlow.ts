/**
 * High/Low Price Overlay
 *
 * Displays horizontal lines marking the highest and lowest prices
 * in the visible viewport, with price labels and percentage change.
 */

export interface ViewportInfo {
  time: {
    start: number;
    end: number;
  };
  price: {
    min: number;
    max: number;
  };
  dimensions: {
    width: number;
    height: number;
    pixelRatio: number;
  };
  visibleBars: number;
  barWidth: number;
}

export class HighLowOverlay {
  private container: HTMLElement;
  private highLine: HTMLDivElement;
  private lowLine: HTMLDivElement;
  private highLabel: HTMLDivElement;
  private lowLabel: HTMLDivElement;
  private overlayContainer: HTMLDivElement;

  constructor(parentElement: HTMLElement) {
    this.container = parentElement;

    // Create overlay container
    this.overlayContainer = document.createElement("div");
    this.overlayContainer.className = "chart-highlow-overlay";
    this.overlayContainer.style.cssText = `
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      pointer-events: none;
      z-index: 100;
    `;

    // Create high line
    this.highLine = document.createElement("div");
    this.highLine.className = "chart-highlow-line high";
    this.highLine.style.cssText = `
      position: absolute;
      left: 0;
      right: 80px;
      border-top: 1px dashed #22c55e;
      opacity: 0.6;
      display: none;
    `;

    // Create low line
    this.lowLine = document.createElement("div");
    this.lowLine.className = "chart-highlow-line low";
    this.lowLine.style.cssText = `
      position: absolute;
      left: 0;
      right: 80px;
      border-top: 1px dashed #ef4444;
      opacity: 0.6;
      display: none;
    `;

    // Create high label
    this.highLabel = document.createElement("div");
    this.highLabel.className = "chart-highlow-label high";
    this.highLabel.style.cssText = `
      position: absolute;
      right: 85px;
      padding: 2px 6px;
      font-size: 10px;
      font-family: 'Menlo', 'Monaco', 'Courier New', monospace;
      background: rgba(34, 197, 94, 0.2);
      border: 1px solid #22c55e;
      border-radius: 3px;
      color: #22c55e;
      font-weight: 600;
      transform: translateY(-50%);
      white-space: nowrap;
      display: none;
    `;

    // Create low label
    this.lowLabel = document.createElement("div");
    this.lowLabel.className = "chart-highlow-label low";
    this.lowLabel.style.cssText = `
      position: absolute;
      right: 85px;
      padding: 2px 6px;
      font-size: 10px;
      font-family: 'Menlo', 'Monaco', 'Courier New', monospace;
      background: rgba(239, 68, 68, 0.2);
      border: 1px solid #ef4444;
      border-radius: 3px;
      color: #ef4444;
      font-weight: 600;
      transform: translateY(-50%);
      white-space: nowrap;
      display: none;
    `;

    // Append to overlay container
    this.overlayContainer.appendChild(this.highLine);
    this.overlayContainer.appendChild(this.lowLine);
    this.overlayContainer.appendChild(this.highLabel);
    this.overlayContainer.appendChild(this.lowLabel);

    // Append to parent
    this.container.appendChild(this.overlayContainer);

    console.log("[HighLowOverlay] Initialized");
  }

  /**
   * Update high/low lines and labels based on viewport
   */
  update(viewport: ViewportInfo | null, currentPrice?: number): void {
    if (!viewport) {
      this.hide();
      return;
    }

    const { price, dimensions } = viewport;
    const { min: lowPrice, max: highPrice } = price;
    const { height } = dimensions;

    // Calculate Y positions (inverted: 0 at top, height at bottom)
    const priceRange = highPrice - lowPrice;
    if (priceRange === 0) {
      this.hide();
      return;
    }

    const highY = ((highPrice - highPrice) / priceRange) * height; // Always 0 (top)
    const lowY = ((highPrice - lowPrice) / priceRange) * height; // Always height (bottom)

    // Position high line and label
    this.highLine.style.top = `${highY}px`;
    this.highLabel.style.top = `${highY}px`;

    // Position low line and label
    this.lowLine.style.top = `${lowY}px`;
    this.lowLabel.style.top = `${lowY}px`;

    // Format labels with price and percentage change
    const formatPrice = (price: number) => {
      return price.toLocaleString("en-US", {
        minimumFractionDigits: 2,
        maximumFractionDigits: 2,
      });
    };

    const formatPercentage = (value: number, reference: number) => {
      if (!reference || reference === 0) return "";
      const percent = ((value - reference) / reference) * 100;
      const sign = percent >= 0 ? "+" : "";
      return ` (${sign}${percent.toFixed(2)}%)`;
    };

    const highText = `H: ${formatPrice(highPrice)}${currentPrice ? formatPercentage(highPrice, currentPrice) : ""}`;
    const lowText = `L: ${formatPrice(lowPrice)}${currentPrice ? formatPercentage(lowPrice, currentPrice) : ""}`;

    this.highLabel.textContent = highText;
    this.lowLabel.textContent = lowText;

    // Show elements
    this.show();
  }

  /**
   * Show overlay
   */
  private show(): void {
    this.highLine.style.display = "block";
    this.lowLine.style.display = "block";
    this.highLabel.style.display = "block";
    this.lowLabel.style.display = "block";
  }

  /**
   * Hide overlay
   */
  private hide(): void {
    this.highLine.style.display = "none";
    this.lowLine.style.display = "none";
    this.highLabel.style.display = "none";
    this.lowLabel.style.display = "none";
  }

  /**
   * Cleanup
   */
  destroy(): void {
    this.overlayContainer.remove();
    console.log("[HighLowOverlay] Destroyed");
  }
}
