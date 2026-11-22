import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { setImmediate } from "node:timers/promises";
import test from "node:test";
import vm from "node:vm";
import ts from "typescript";

// Exercise current TS controller code in memory, with no browser, build output,
// HTTP requests, credentials, or backend writes. UI5 objects are isolated stubs.
const modules = new Map();
const errors = [];

class JSONModel {
  constructor(data = {}) {
    this.data = data;
  }
  getData() {
    return this.data;
  }
  setData(data) {
    this.data = data;
  }
}

class Controller {
  constructor() {
    this.feedback = [];
    this.busy = [];
    this.routes = new Map();
    this.models = new Map();
    this.control = { setBusy: (value) => this.busy.push(value) };
    this.view = {
      addStyleClass() {},
      addEventDelegate() {},
      isDestroyed: () => false,
      setModel: (model, name) => {
        this.models.set(name, model);
        return this.view;
      },
      getModel: (name) => this.models.get(name),
      byId: () => this.control
    };
    this.component = {
      getContentDensityClass: () => "sapUiSizeCozy",
      getResourceBundle: () => ({ getText: (key) => key }),
      getRouter: () => ({
        getRoute: (name) => ({
          attachMatched: (callback, context) => this.routes.set(name, callback.bind(context)),
          attachPatternMatched: (callback, context) => this.routes.set(name, callback.bind(context))
        })
      })
    };
  }
  getView() {
    return this.view;
  }
  getOwnerComponent() {
    return this.component;
  }
  byId() {
    return this.control;
  }
}

function loadController(name) {
  if (modules.has(name)) return modules.get(name);
  const filename = new URL(`../webapp/controller/${name}.controller.ts`, import.meta.url);
  const { outputText } = ts.transpileModule(readFileSync(filename, "utf8"), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022, esModuleInterop: true }
  });
  const module = { exports: {} };
  const require = (id) => {
    if (id === "sap/ui/core/mvc/Controller") return Controller;
    if (id === "sap/ui/model/json/JSONModel") return JSONModel;
    if (id === "jquery") return () => ({ addClass() {} });
    if (id === "sap/m/MessageToast") return { show() {} };
    if (id === "sap/m/GroupHeaderListItem" || id === "sap/ui/model/Filter") return class {};
    if (id === "sap/ui/model/FilterOperator") return {};
    if (id === "./Base.controller") return loadController("Base");
    if (id === "./BaseTable.controller") {
      return class extends loadController("Base") {
        init(table) {
          this.table = table;
        }
        sortTable() {}
      };
    }
    // Detail-controller navigation constants; no table controller is executed.
    if (id === "./HeatsTable.controller") return { HEAT_MODEL: "heat", HEAT_NAV_MODEL: "heatNav" };
    if (id === "./RacesTable.controller") return { RACE_MODEL: "race", RACE_NAV_MODEL: "raceNav" };
    if (id === "../model/Formatter") return {};
    if (id === "sap/ui/core/routing/History") return {};
    throw new Error(`Unmocked controller dependency: ${id}`);
  };
  vm.runInNewContext(
    outputText,
    {
      module,
      exports: module.exports,
      require,
      console: { error: (...args) => errors.push(args) }
    },
    { filename: filename.pathname }
  );
  const controller = module.exports.default;
  modules.set(name, controller);
  return controller;
}

const BaseController = loadController("Base");
BaseController.prototype.showDataUpdatedMessage = function (succeeded) {
  this.feedback.push(succeeded);
};

test("runAsync handles a rejection once and does not treat false as a rejection", async () => {
  const controller = new BaseController();
  const error = new Error("Synthetic load failure");
  errors.length = 0;
  assert.equal(controller.runAsync(Promise.reject(error), "Load failed"), undefined);
  await setImmediate();
  assert.deepEqual(controller.feedback, [false]);
  assert.equal(errors.length, 1);
  assert.equal(errors[0][1], error);
  controller.runAsync(Promise.resolve(false), "Load failed");
  await setImmediate();
  assert.deepEqual(controller.feedback, [false]);
});

test("late rejections do not notify a destroyed view", async () => {
  const controller = new BaseController();
  controller.view.isDestroyed = () => true;
  controller.runAsync(Promise.reject(new Error("Late failure")), "Load failed");
  await setImmediate();
  assert.deepEqual(controller.feedback, []);
});

test("statistics callbacks return void and clear busy state on refresh failure", async () => {
  const controller = new (loadController("Statistics"))();
  controller.loadStatistics = () => Promise.reject(new Error("Synthetic statistics failure"));
  controller.onInit();
  assert.equal(controller.routes.get("statistics")({}), undefined);
  await setImmediate();
  assert.deepEqual(controller.feedback, [false]);
  controller.feedback.length = 0;
  assert.equal(controller.onRefreshButtonPress({}), undefined);
  assert.deepEqual(controller.busy, [true]);
  await setImmediate();
  assert.deepEqual(controller.busy, [true, false]);
  assert.deepEqual(controller.feedback, [false]);
});

test("heat route arguments are consumed synchronously, before a pooled event is cleared", async () => {
  const controller = new (loadController("HeatDetails"))();
  const loads = [];
  controller.updateNavOnPatternMatched = () => {};
  controller.loadHeatModel = () => {
    loads.push(controller.heatId);
    return Promise.resolve(true);
  };
  controller.onInit();
  const event = { getParameter: () => ({ heatId: 47 }) };
  assert.equal(controller.routes.get("heatDetails")(event), undefined);
  event.getParameter = () => {
    throw new Error("Event was already cleared");
  };
  assert.deepEqual(loads, [47]);
  await setImmediate();
  assert.deepEqual(controller.feedback, []);
});

test("club athlete filters are applied after loading, with a captured club ID", async () => {
  const controller = new (loadController("AthletesTable"))();
  let resolve;
  const loaded = new Promise((done) => {
    resolve = done;
  });
  const filters = [];
  controller.loadModel = () => loaded;
  controller.applyClubFilter = (id) => filters.push(id);
  controller.onInit();
  const event = { getParameter: () => ({ clubId: 7 }) };
  assert.equal(controller.routes.get("clubAthletes")(event), undefined);
  event.getParameter = () => {
    throw new Error("Event was already cleared");
  };
  assert.deepEqual(filters, []);
  resolve(true);
  await setImmediate();
  assert.deepEqual(filters, [7]);
});

test("race refresh restores the disabled button even when loading rejects", async () => {
  const controller = new (loadController("RaceDetails"))();
  const enabled = [];
  controller.loadRaceModel = () => Promise.reject(new Error("Synthetic race failure"));
  controller.onRefreshButtonPress({ getSource: () => ({ setEnabled: (value) => enabled.push(value) }) });
  assert.deepEqual(enabled, [false]);
  await setImmediate();
  assert.deepEqual(enabled, [false, true]);
  assert.deepEqual(controller.feedback, [false]);
});

// XML bindings are runtime references, so TypeScript alone cannot detect an
// accidental visibility reduction of their inherited handlers or formatters.
function visibilityModel() {
  const directory = new URL("../webapp/controller/", import.meta.url);
  const byModule = new Map();
  const byClass = new Map();
  const files = readdirSync(directory).filter((file) => file.endsWith(".controller.ts"));
  for (const file of [...files, "../Component.ts", "../model/Formatter.ts"]) {
    const source = ts.createSourceFile(
      file,
      readFileSync(new URL(file, directory), "utf8"),
      ts.ScriptTarget.Latest,
      true
    );
    const declaration = source.statements.find(ts.isClassDeclaration);
    assert.ok(declaration, file);
    const moduleName = file.replace(/\.controller\.ts$/, "");
    byModule.set(moduleName, declaration);
    byClass.set(declaration.name.text, declaration);
  }
  const member = (declaration, name) => {
    const own = declaration.members.find((item) => item.name?.getText() === name);
    if (own) return own;
    const parent = declaration.heritageClauses?.find((clause) => clause.token === ts.SyntaxKind.ExtendsKeyword)
      ?.types[0];
    const base = parent && byClass.get(parent.expression.getText());
    return base && member(base, name);
  };
  const visibility = (item) => {
    assert.ok(item, "member must exist");
    if (item.modifiers?.some((modifier) => modifier.kind === ts.SyntaxKind.PrivateKeyword)) return "private";
    if (item.modifiers?.some((modifier) => modifier.kind === ts.SyntaxKind.ProtectedKeyword)) return "protected";
    return "public";
  };
  return { byModule, byClass, member, visibility };
}

test("subclass helpers and table hooks are protected, while component APIs stay public", () => {
  const { byModule, byClass, member, visibility } = visibilityModel();
  const base = byModule.get("Base");
  for (const name of [
    "showDataUpdatedMessage",
    "showInfoMessageToast",
    "showErrorMessageToast",
    "runAsync",
    "getContentDensityClass",
    "getEventBus",
    "getComponentJSONModel",
    "getRouter",
    "getViewJSONModel",
    "setViewModel",
    "navBack",
    "i18n",
    "updateJSONModel",
    "navToStartPage",
    "navToRaceDetails",
    "navToHeatDetails",
    "navToClubDetails",
    "navToClubAthletes",
    "navToAthleteDetails",
    "getFilters",
    "getActiveRegatta"
  ]) {
    assert.equal(visibility(member(base, name)), "protected", name);
  }
  const table = byModule.get("BaseTable");
  for (const name of [
    "init",
    "getViewSettingsDialog",
    "setSearchFilters",
    "clearFilters",
    "applyFilters",
    "sortTable",
    "onItemChanged"
  ]) {
    assert.equal(visibility(member(table, name)), "protected", name);
  }
  for (const name of ["AthletesTable", "ClubsTable", "HeatsTable", "RacesTable", "Timekeeping"]) {
    assert.equal(visibility(member(byModule.get(name), "onItemChanged")), "protected", name);
  }
  for (const name of ["getActiveRegatta", "getFilters", "getContentDensityClass", "getResourceBundle", "init"]) {
    assert.equal(visibility(member(byClass.get("Component"), name)), "public", name);
  }
});

test("XML-bound controller methods and formatter APIs retain public visibility", () => {
  const { byModule, byClass, member, visibility } = visibilityModel();
  const directory = new URL("../webapp/view/", import.meta.url);
  let checked = 0;
  for (const file of readdirSync(directory).filter((file) => file.endsWith(".xml"))) {
    const xml = readFileSync(new URL(file, directory), "utf8");
    const controllerName = xml
      .match(/\bcontrollerName="([^"]+)"/)?.[1]
      .split(".")
      .at(-1);
    const controller = controllerName && byModule.get(controllerName);
    const handlers = new Set([
      ...Array.from(xml.matchAll(/\b[\w:]+\s*=\s*["']\.([\w$]+)["']/g), (match) => match[1]),
      ...Array.from(
        xml.matchAll(/(?:formatter|group|groupHeaderFactory)\s*:\s*['"]\.([\w$.]+)['"]/g),
        (match) => match[1]
      )
    ]);
    for (const path of handlers) {
      if (path.startsWith("formatter.")) {
        if (controller) assert.equal(visibility(member(controller, "formatter")), "public", `${file}: formatter`);
        assert.equal(
          visibility(member(byClass.get("Formatter"), path.slice("formatter.".length))),
          "public",
          `${file}: ${path}`
        );
      } else if (controller) {
        assert.equal(visibility(member(controller, path)), "public", `${file}: ${path}`);
      } else {
        // Fragments receive their controller from their caller. All matching
        // app declarations must remain public (including base-table handlers).
        const candidates = [...byClass.values()].flatMap((declaration) =>
          declaration.members.filter((item) => item.name?.getText() === path)
        );
        assert.ok(candidates.length, `${file}: ${path}`);
        for (const candidate of candidates) assert.equal(visibility(candidate), "public", `${file}: ${path}`);
      }
      checked++;
    }
  }
  assert.ok(checked > 50, "expected broad coverage of view/fragment bindings");
});

test("the notifications route loads the Notifications view and controller", () => {
  const manifest = JSON.parse(readFileSync(new URL("../webapp/manifest.json", import.meta.url), "utf8"));
  const routing = manifest["sap.ui5"].routing;
  const route = routing.routes.find((item) => item.name === "notifications");
  assert.equal(route.pattern, "notifications");
  assert.equal(route.target, "notifications");
  assert.ok(!routing.routes.some((item) => item.name === "admin" || item.pattern === "admin"));
  assert.equal(routing.targets.admin, undefined);
  assert.equal(routing.targets[route.target].name, "Notifications");
  const launchpad = readFileSync(new URL("../webapp/view/Launchpad.view.xml", import.meta.url), "utf8");
  assert.match(launchpad, /press="\.navToNotifications"/);
  const view = readFileSync(new URL("../webapp/view/Notifications.view.xml", import.meta.url), "utf8");
  assert.match(view, /controllerName="de\.regatta_hd\.infoportal\.controller\.Notifications"/);
  const controller = readFileSync(new URL("../webapp/controller/Notifications.controller.ts", import.meta.url), "utf8");
  assert.match(controller, /export default class NotificationsController extends BaseController/);
  assert.ok(!readdirSync(new URL("../webapp/view/", import.meta.url)).includes("Admin.view.xml"));
  assert.ok(!readdirSync(new URL("../webapp/controller/", import.meta.url)).includes("Admin.controller.ts"));
});
