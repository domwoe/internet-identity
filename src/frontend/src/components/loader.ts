import { html, render } from "lit-html";

const loader = () => html`<style>
    #loader {
      position: fixed;
      z-index: var(--vz-loader);
      top: 0;
      left: 0;
      width: 100vw;
      height: 100vh;
      background: rgba(0, 0, 0, 0.75);
      display: flex;
      justify-content: center;
      align-items: center;
    }
    #loader img {
      width: 125px;
      min-width: 125px;
      max-width: calc(100vw - 1rem);
      margin: auto;
      display: block;
    }
  </style>
  <picture id="loader">
    <img src="/loader.webp" alt="loading" />
  </picture>`;

const startLoader = () => {
  const container = document.getElementById("loaderContainer") as HTMLElement;
  render(loader(), container);
};

const endLoader = () => {
  const container = document.getElementById("loaderContainer") as HTMLElement;
  render(html``, container);
};

export const withLoader = async <A>(
  action: () => Promise<A>,
  showLoader = true
): Promise<A> => {
  document.body.classList.add("loading");

  if (showLoader) startLoader();
  try {
    return await action();
  } finally {
    document.body.classList.remove("loading");
    if (showLoader) endLoader();
  }
};
