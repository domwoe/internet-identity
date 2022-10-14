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
  const chasmToggle = () =>
    withRef(chasmRef, (chasm) => {
      const classes = chasm.classList;

      if (classes.contains("c-chasm--closed")) {
        classes.remove("c-chasm--closed");
        classes.add("c-chasm--open");

        withRef(chasmToggleRef, (arrow) =>
          arrow.classList.add("c-chasm__button--flipped")
        );
      } else if (classes.contains("c-chasm--open")) {
        classes.remove("c-chasm--open");
        classes.add("c-chasm--closed");

        withRef(chasmToggleRef, (arrow) =>
          arrow.classList.remove("c-chasm__button--flipped")
        );
      }
    });

  const template = html` <div class="l-stack">
    <ul class="c-list c-list--anchors">
      ${props.savedAnchors.map(
        (anchor: bigint) => html` <li
          class="c-list__item c-list__item--icon icon-trigger"
          @click="${() => props.onSubmit(anchor)}"
        >
          <span class="c-list__parcel"> ${anchor} </span>
          <i class="c-list__icon"> ${arrowRight} </i>
        </li>`
      )}
      <div class="t-centered l-stack">
        <span class="t-action" @click="${chasmToggle}"
          >Use another anchor
          <span ${ref(chasmToggleRef)} class="t-link__icon c-chasm-button"
            >${caretDownIcon}
          </span>
        </span>
      </div>
      <div ${ref(chasmRef)} class="c-chasm c-chasm--closed l-stack">
        ${anchorInput.template}
        <button class="c-button" @click="${anchorInput.submit}">
          Oh ouiiii
        </button>
      </div>
    </ul>
  </div>`;

  return { template };
};
