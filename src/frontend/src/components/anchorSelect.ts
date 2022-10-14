import { html, TemplateResult } from "lit-html";
import { caretDownIcon, arrowRight } from "./icons";
import { withRef } from "../utils/utils";
import { createRef, ref, Ref } from "lit-html/directives/ref.js";
import { mkAnchorInput } from "./anchorInput";

/** A component for inputting an anchor number */
export const mkAnchorSelect = (props: {
  savedAnchors: bigint[];
  onSubmit: (userNumber: bigint) => void; // TODO: rename to onPick
}): {
  template: TemplateResult;
} => {
  const anchorInput = mkAnchorInput({
    inputId: "bad-id",
    onSubmit: props.onSubmit,
  });

  /* the chasm that opens to reveal custom anchor selection */
  const chasmRef: Ref<HTMLDivElement> = createRef();

  /* the (purely visual) arrow on the chasm */
  const chasmToggleRef: Ref<HTMLSpanElement> = createRef();

  /* Toggle the chasm open/closed */
  const chasmOpen = () =>
    withRef(chasmRef, (chasm) => {
      const classes = chasm.classList;
      classes.add("c-list__item--focus");
    });

  const template = html` <div class="l-stack">
    <ul class="c-list c-list--anchors">
      ${props.savedAnchors.map(
        (anchor: bigint) => html` <li
          class="c-list__item c-list__item--vip c-list__item--icon icon-trigger"
        >
          <button
            class="c-list__parcel"
            @click="${() => props.onSubmit(anchor)}"
            tabindex="0"
          >
            ${anchor}
          </button>
          <i class="c-list__icon"> ${arrowRight} </i>
        </li>`
      )}
      <li
        class="c-list__item c-list__item--noFocusStyle"
        @focusin="${chasmOpen}"
        ${ref(chasmRef)}
      >
        <a
          class="c-list__parcel c-list__parcel--fullwidth c-list__parcel--summary"
          href="#otheranchor"
          @click="${chasmOpen}"
          tabindex="-1"
        >
          Use an other anchor
        </a>
        <div
          id="#otheranchor"
          class="c-list__parcel c-list__parcel--detail c-list__parcel--fullwidth"
        >
          ${anchorInput.template}
          <button class="c-button" @click="${anchorInput.submit}">
            Oh ouiiii
          </button>
          <p class="l-stack">
            An <b class="t-strong">Identity Anchor</b> is a
            <b class="t-strong">unique ID</b> that is used to authenticate
            yourself. You will be able to use it to
            <b class="t-strong">log in</b> to all kinds of apps.
          </p>
          <div class="l-stack">
            <button class="c-button c-button--secondary">Create a new anchor</a>
          </div>
        </div>
      </li>
    </ul>
  </div>`;

  return { template };
};
