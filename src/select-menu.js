let selectMenuSequence = 0;
let globalHandlersBound = false;

function accessibleNameFor(select) {
  return select.getAttribute("aria-label")
    || select.closest("label")?.querySelector(".field-label, :scope > span")?.textContent?.trim()
    || "";
}

function optionButtons(control) {
  return [...control.querySelectorAll(".select-menu-option:not(:disabled)")];
}

function closeSelectMenu(control, { restoreFocus = false } = {}) {
  const trigger = control?.querySelector(".select-menu-trigger");
  const popover = control?.querySelector(".select-menu-popover");
  if (!trigger || !popover) return;
  popover.classList.add("hidden");
  popover.classList.remove("open-up");
  trigger.setAttribute("aria-expanded", "false");
  if (restoreFocus) trigger.focus();
}

function closeOtherSelectMenus(current) {
  document.querySelectorAll(".select-menu-control").forEach((control) => {
    if (control !== current) closeSelectMenu(control);
  });
}

function openSelectMenu(control, { focusSelected = false } = {}) {
  const trigger = control.querySelector(".select-menu-trigger");
  const popover = control.querySelector(".select-menu-popover");
  if (!trigger || !popover || trigger.disabled) return;
  closeOtherSelectMenus(control);
  refreshSelectMenu(control);
  popover.classList.remove("hidden", "open-up");
  trigger.setAttribute("aria-expanded", "true");

  const triggerRect = trigger.getBoundingClientRect();
  const popoverHeight = popover.getBoundingClientRect().height;
  const roomBelow = window.innerHeight - triggerRect.bottom;
  if (roomBelow < popoverHeight + 8 && triggerRect.top > roomBelow) {
    popover.classList.add("open-up");
  }
  if (focusSelected) {
    (popover.querySelector('[aria-selected="true"]') ?? optionButtons(control)[0])?.focus();
  }
}

function chooseOption(control, optionButton) {
  const select = control.previousElementSibling;
  if (!(select instanceof HTMLSelectElement) || optionButton.disabled) return;
  const changed = select.value !== optionButton.dataset.value;
  select.value = optionButton.dataset.value;
  refreshSelectMenu(control);
  closeSelectMenu(control, { restoreFocus: true });
  if (changed) {
    select.dispatchEvent(new Event("input", { bubbles: true }));
    select.dispatchEvent(new Event("change", { bubbles: true }));
  }
}

function moveOptionFocus(control, direction) {
  const buttons = optionButtons(control);
  if (!buttons.length) return;
  const current = buttons.indexOf(document.activeElement);
  let next = current;
  if (direction === "first") next = 0;
  else if (direction === "last") next = buttons.length - 1;
  else if (direction === 1) next = current < 0 ? 0 : (current + 1) % buttons.length;
  else next = current <= 0 ? buttons.length - 1 : current - 1;
  buttons[next]?.focus();
}

function bindGlobalHandlers() {
  if (globalHandlersBound) return;
  globalHandlersBound = true;
  document.addEventListener("click", (event) => {
    if (event.target.closest(".select-menu-control")) return;
    closeOtherSelectMenus(null);
  });
  document.addEventListener("keydown", (event) => {
    if (event.key !== "Escape") return;
    const openControl = document.querySelector(
      '.select-menu-trigger[aria-expanded="true"]',
    )?.closest(".select-menu-control");
    if (openControl) {
      event.preventDefault();
      closeSelectMenu(openControl, { restoreFocus: true });
    }
  });
}

function createControl(select) {
  const id = `select-menu-${++selectMenuSequence}`;
  const control = document.createElement("div");
  control.className = "select-menu-control";
  control.dataset.selectId = select.id || select.name || id;

  const trigger = document.createElement("button");
  trigger.type = "button";
  trigger.className = "btn btn-default select-menu-trigger";
  trigger.setAttribute("aria-expanded", "false");
  trigger.setAttribute("aria-haspopup", "listbox");
  trigger.setAttribute("aria-controls", id);
  const accessibleName = accessibleNameFor(select);
  if (accessibleName) trigger.dataset.accessibleName = accessibleName;

  const label = document.createElement("span");
  label.className = "select-menu-current";
  const chevron = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  chevron.setAttribute("viewBox", "0 0 12 12");
  chevron.setAttribute("fill", "none");
  chevron.setAttribute("stroke", "currentColor");
  chevron.setAttribute("stroke-width", "1.5");
  chevron.setAttribute("aria-hidden", "true");
  const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
  path.setAttribute("d", "m3 4.5 3 3 3-3");
  path.setAttribute("stroke-linecap", "round");
  path.setAttribute("stroke-linejoin", "round");
  chevron.append(path);
  trigger.append(label, chevron);

  const popover = document.createElement("div");
  popover.id = id;
  popover.className = "select-menu-popover hidden";
  popover.setAttribute("role", "listbox");
  control.append(trigger, popover);
  select.insertAdjacentElement("afterend", control);
  select.classList.add("select-menu-native");
  select.setAttribute("aria-hidden", "true");
  select.tabIndex = -1;
  select.hidden = true;

  trigger.addEventListener("click", (event) => {
    event.preventDefault();
    event.stopPropagation();
    if (trigger.getAttribute("aria-expanded") === "true") closeSelectMenu(control);
    else openSelectMenu(control);
  });
  trigger.addEventListener("keydown", (event) => {
    if (!["ArrowDown", "ArrowUp", "Enter", " "].includes(event.key)) return;
    event.preventDefault();
    openSelectMenu(control, { focusSelected: true });
  });
  popover.addEventListener("click", (event) => {
    const option = event.target.closest(".select-menu-option");
    if (!option) return;
    event.preventDefault();
    event.stopPropagation();
    chooseOption(control, option);
  });
  popover.addEventListener("keydown", (event) => {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      moveOptionFocus(control, event.key === "ArrowDown" ? 1 : -1);
    } else if (event.key === "Home" || event.key === "End") {
      event.preventDefault();
      moveOptionFocus(control, event.key === "Home" ? "first" : "last");
    } else if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      const option = event.target.closest(".select-menu-option");
      if (option) chooseOption(control, option);
    }
  });
  select.addEventListener("change", () => refreshSelectMenu(control));
  return control;
}

function refreshSelectMenu(control) {
  const select = control.previousElementSibling;
  if (!(select instanceof HTMLSelectElement)) return;
  const trigger = control.querySelector(".select-menu-trigger");
  const current = control.querySelector(".select-menu-current");
  const popover = control.querySelector(".select-menu-popover");
  const selected = select.selectedOptions[0] ?? select.options[0];
  current.textContent = selected?.textContent?.trim() ?? "";
  const accessibleName = accessibleNameFor(select);
  if (accessibleName) trigger.dataset.accessibleName = accessibleName;
  if (trigger.dataset.accessibleName) {
    trigger.setAttribute(
      "aria-label",
      `${trigger.dataset.accessibleName}: ${current.textContent}`,
    );
  }
  trigger.disabled = select.disabled;
  popover.replaceChildren();

  for (const option of select.options) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "select-menu-option";
    button.dataset.value = option.value;
    button.setAttribute("role", "option");
    button.setAttribute("aria-selected", String(option.selected));
    button.disabled = option.disabled;
    const text = document.createElement("span");
    text.textContent = option.textContent?.trim() ?? "";
    const check = document.createElement("span");
    check.className = "select-menu-check";
    check.textContent = option.selected ? "✓" : "";
    check.setAttribute("aria-hidden", "true");
    button.append(text, check);
    popover.append(button);
  }
}

export function enhanceSelectMenus(root = document) {
  bindGlobalHandlers();
  root.querySelectorAll("select.field-select:not(.select-menu-native)").forEach((select) => {
    const control = createControl(select);
    refreshSelectMenu(control);
  });
}

export function refreshSelectMenus(root = document) {
  root.querySelectorAll(".select-menu-control").forEach(refreshSelectMenu);
}
