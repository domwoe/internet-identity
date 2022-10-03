import { remote } from "webdriverio";
import { existsSync, mkdirSync } from "fs";

async function main() {
  await withChrome(takeShowcaseScreenshots);
}

/** Open each showcase page one after the other and screenshot it */
async function takeShowcaseScreenshots(browser: WebdriverIO.Browser) {
  await visit(browser, "http://localhost:8080/");

  // The landing page has a link for every page. The link tags have `data-page-name`
  // attributes, which we gather as the list of page names.
  const pageLinks = await browser.$$("[data-page-name]");
  const pageNames = await Promise.all(
    pageLinks.map(async (link) => {
      const pageName = await link.getAttribute("data-page-name");
      return pageName;
    })
  );

  // Set up the directory where we'll save the screenshots
  const screenshotsDir =
    process.env["SCREENSHOTS_DIR"] ?? "./screenshots/custom";
  if (!existsSync(screenshotsDir)) {
    mkdirSync(screenshotsDir, { recursive: true });
  }

  // Iterate the pages and screenshot them
  for (const pageName of pageNames) {
    // Skip the loader, because it's animated
    if (pageName === "loader") {
      continue;
    }

    // In the case of the faq we modify the URL slightly to show an open entry
    if (pageName === "faq") {
      await visit(browser, `http://localhost:8080/${pageName}#lost-device`);
    } else {
      await visit(browser, `http://localhost:8080/${pageName}`);
    }

    await browser.execute('document.body.style.caretColor = "transparent"');
    await browser.saveScreenshot(`${screenshotsDir}/${pageName}.png`);
  }
}

/** Create a chrome instance and run callback, deleting session afterwards */
async function withChrome<T>(
  cb: (browser: WebdriverIO.Browser) => T
): Promise<T> {
  // Screenshot image dimension, if specified
  const { windowSize, deviceName } = readScreenshotsConfig();

  const browser = await remote({
    capabilities: {
      browserName: "chrome",
      "goog:chromeOptions": {
        args: [
          "headless",
          "disable-gpu",
          ...(windowSize !== undefined ? [`--window-size=${windowSize}`] : []),
        ],
        ...(deviceName !== undefined
          ? { mobileEmulation: { deviceName } }
          : {}),
      },
    },
  });

  const res = await cb(browser);
  await browser.deleteSession();
  return res;
}

/** Visit page and wait until loaded */
async function visit(browser: WebdriverIO.Browser, url: string) {
  await browser.url(url);
  await browser.waitUntil(
    () => browser.execute(() => document.readyState === "complete"),
    {
      timeout: 10 * 1000,
      timeoutMsg: "Browser did not load after 10 seconds",
    }
  );

  await browser.execute(() => {
    const notransition = `
            * {
              -webkit-transition: none !important;
              -moz-transition: none !important;
              -o-transition: none !important;
              transition: none !important;
            }
        `;
    const style = document.createElement("style");
    style.type = "text/css";
    style.appendChild(document.createTextNode(notransition));
    document.getElementsByTagName("head")[0].appendChild(style);
  });

  await browser.pause(1000);
}

/**
 * Read the screenshots configuration based on 'SCREENSHOTS_TYPE'
 * (either 'mobile' or 'desktop') and returns the appropriate device
 * name and/or window size.
 *
 * NOTE: the window size is only necessary due to a bug in webdriverio:
 * * https://github.com/webdriverio/webdriverio/issues/8903
 */
function readScreenshotsConfig(): { windowSize?: string; deviceName?: string } {
  const screenshotsType = process.env["SCREENSHOTS_TYPE"];
  switch (screenshotsType) {
    case "mobile":
      return { windowSize: "360,667", deviceName: "iPhone SE" };
      break;
    case undefined:
      return {};
      break;
    case "desktop":
      return {};
      break;
    default:
      throw Error("Unknown screenshots type: " + screenshotsType);
      break;
  }
}

main();
