import { html, render } from "lit-html";
import { wrenchIcon } from "../components/icons";
import { features } from "../features";

export const postUrl = "#";

/** Block if the "IS_MIGRATING" feature is set.
 * Can be jailbroken by clicking the icon 3 times.
 **/
export function blockIfMigrating(): Promise<void> {
  return new Promise<void>((resolve) => {
    if (!features.IS_MIGRATING) {
      resolve();
      return;
    }

    let jailbreakHitCount = 0;
    const jailbreakHit = () => {
      if (++jailbreakHitCount >= 3) {
        resolve();
      }
    };
    render(
      html`
        <div class="l-container c-card c-card--highlight">
          <h1 class="t-title t-title--main">
            Maintenance <i @click="${jailbreakHit}">${wrenchIcon}</i>
          </h1>
          <div class="l-stack">
            <h2 class="t-title">Internet Identity will be back shortly.</h2>
            <p class="t-paragraph">
              Internet Identity is undergoing a
              <a
                class="t-link"
                target="_blank"
                rel="noopener noreferrer"
                href="${postUrl}"
                >migration</a
              >. You can still use Internet Identity to authenticate to dapps,
              but anchor management and anchor creation is temporarily disabled.
            </p>
            <p class="t-paragraph">
              For frequently asked questions, check the
              <a href="/faq" title="Go to the Internet Identity FAQ page"
                >FAQ page</a
              >.
            </p>
          </div>
        </div>
      `,
      document.getElementById("pageContent") as HTMLElement
    );
  });
}
