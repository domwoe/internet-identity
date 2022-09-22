import { remote } from "webdriverio";
import { spawn } from "child_process";
import { get } from "http";

/** Create a chrome instance and run callback, deleting session afterwards */
async function withChrome<T>(
  cb: (browser: WebdriverIO.Browser) => T
): Promise<T> {
  const browser = await remote({
    capabilities: {
      browserName: "chrome",
      "goog:chromeOptions": {
        args: ["headless", "disable-gpu"],
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
}

async function takeShowcaseScreenshots(browser: WebdriverIO.Browser) {
  await visit(browser, "http://localhost:8080/");

  const pageLinks = await browser.$$("[data-page-name]");
  const pageNames = await Promise.all(
    pageLinks.map(async (link) => {
      const pageName = await link.getAttribute("data-page-name");
      return pageName;
    })
  );

  // Ensure bliking cursors don't mess up screenshots
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
    await browser.saveScreenshot(`./screenshots/${pageName}.png`);
  }
}

async function main() {
  await withChrome(takeShowcaseScreenshots);
}

main();
