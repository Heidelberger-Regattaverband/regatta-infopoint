import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
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
