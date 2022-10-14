import { html, TemplateResult } from "lit-html";
import { arrowRight } from "./icons";
import { until } from "lit-html/directives/until.js";
import { mkAnchorInput } from "./anchorInput";

/** A component for inputting an anchor number */
export const mkAnchorSelect = (props: {
  savedAnchors: bigint[];
  onSubmit: (userNumber: bigint) => void; // TODO: rename to onPick
}): {
  template: TemplateResult;
} => {
  // TODO: remove first l-stack
  // TODO: add focus to first elem
  const template = html` <ul class="c-list c-list--anchors l-stack">
    ${elements({
      savedAnchors: props.savedAnchors,
      onSubmit: props.onSubmit,
    })}
  </ul>`;

  return { template };
};

function elements(props: {
  savedAnchors: bigint[];
  onSubmit: (userNumber: bigint) => void;
}): TemplateResult[] {
  const otherAnchorMenuTpl = otherAnchorMenu({ onSubmit: props.onSubmit });
  const savedAnchorsTpl = props.savedAnchors.map((anchor) =>
    anchorItem({ anchor, onSubmit: props.onSubmit })
  );

  let elems = [];

  if (savedAnchorsTpl.length > 0) {
    // Function that replaces the "other anchor" button with the "other anchor" menu
    // (actually a reference written when the template is created)
    const ptr: { otherAnchorOpen?: () => void } = {};

    const otherAnchorOffer: TemplateResult = html`
      <button
        class="t-link c-list__parcel c-list__parcel--fullwidth c-list__parcel--summary"
        @click="${() => {
          ptr.otherAnchorOpen?.();
        }}"
        @focus="${() => {
          ptr.otherAnchorOpen?.();
        }}"
      >
        Use another anchor<i class="c-list__icon"> … </i>
      </button>
    `;

    elems = savedAnchorsTpl;
    elems.push(html` <li class="c-list__item c-list__item--noFocusStyle">
      ${until(
        // replace the "use another anchor" button with actual menu when ready (clicked)
        new Promise<void>((resolve) => {
          ptr.otherAnchorOpen = resolve;
        }).then(() =>
          Promise.resolve(otherAnchorMenu({ onSubmit: props.onSubmit }))
        ),
        otherAnchorOffer
      )}
    </li>`);
  } else {
    elems.push(otherAnchorMenuTpl);
  }

  return elems;
}

const anchorItem = (props: {
  anchor: bigint;
  onSubmit: (userNumber: bigint) => void;
}): TemplateResult => html`
  <li class="c-list__item c-list__item--vip c-list__item--icon icon-trigger">
    <button
      class="c-list__parcel"
      @click="${() => props.onSubmit(props.anchor)}"
      tabindex="0"
    >
      ${props.anchor}
    </button>
    <i class="c-list__icon"> ${arrowRight} </i>
  </li>
`;

function otherAnchorMenu(props: {
  onSubmit: (userNumber: bigint) => void;
}): TemplateResult {
  const anchorInput = mkAnchorInput({
    inputId: "bad-id",
    onSubmit: props.onSubmit,
  });

  return html`
    <div
      class="c-list__parcel c-list__parcel--fullwidth c-list__parcel--detail"
    >
      ${anchorInput.template}
      <button class="c-button" @click="${anchorInput.submit}">Oh ouiiii</button>
      <p class="l-stack">
        An <b class="t-strong">Identity Anchor</b> is a
        <b class="t-strong">unique ID</b> that is used to authenticate yourself.
        You will be able to use it to <b class="t-strong">log in</b> to all
        kinds of apps.
      </p>
      <div class="l-stack">
        <button class="c-button c-button--secondary">
          Create a new anchor
        </button>
      </div>
    </div>
  `;
}
