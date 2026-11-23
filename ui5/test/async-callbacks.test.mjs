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
const ajaxRequests = [];
const toasts = [];
const confirmations = [];
const fakeJQuery = () => ({ addClass() {} });
fakeJQuery.ajax = (options) => {
  let resolve;
  let reject;
  const completed = new Promise((done, failed) => {
    resolve = done;
    reject = failed;
  });
  const request = {
    ...options,
    // jqXHR is thenable; awaiting it resolves with the response data.
    then: completed.then.bind(completed),
    settled: false,
    succeed(data) {
      assert.equal(this.settled, false, "requests settle once");
      this.settled = true;
      options.success?.(data);
      resolve(data);
    },
    fail(error = { status: 503, statusText: "Synthetic request failure" }) {
      assert.equal(this.settled, false, "requests settle once");
      this.settled = true;
      options.error?.(error);
      reject(error);
    }
  };
  ajaxRequests.push(request);
  return request;
};
const routesModule = {};
const routesSource = readFileSync(new URL("../webapp/model/Routes.ts", import.meta.url), "utf8");
vm.runInNewContext(
  ts.transpileModule(routesSource, {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 }
  }).outputText,
  { exports: routesModule }
);
const { Routes } = routesModule;

const leafletState = { maps: [] };
const fakeLeaflet = {
  latLng: (lat, lng) => ({ lat, lng }),
  latLngBounds: (points) => ({ points, isValid: () => points.length > 0 }),
  layerGroup: (layers = []) => ({
    layers: [...layers],
    clearLayers() {
      this.layers = [];
      return this;
    },
    addLayer(layer) {
      this.layers.push(layer);
      return this;
    },
    eachLayer(callback) {
      this.layers.forEach(callback);
      return this;
    }
  }),
  tileLayer: (url) => ({ url }),
  circle: (position, options) => ({ position, options }),
  icon: (options) => options,
  popup: () => ({
    setContent(content) {
      this.content = content;
      return this;
    }
  }),
  marker: (position) => ({
    position,
    bindPopup(popup) {
      this.popup = popup;
      return this;
    },
    setIcon(icon) {
      this.icon = icon;
      return this;
    },
    getLatLng: () => position,
    openPopup() {}
  }),
  control: { layers: () => ({ addTo() {} }), scale: () => ({ addTo() {} }) },
  map: (container, options) => {
    const instance = {
      container,
      options,
      fitted: [],
      focused: [],
      removed: 0,
      resized: 0,
      getContainer: () => container,
      fitBounds(bounds) {
        this.fitted.push(bounds);
      },
      setView(position, zoom) {
        this.focused.push({ position, zoom });
      },
      invalidateSize() {
        this.resized++;
      },
      remove() {
        this.removed++;
      }
    };
    leafletState.maps.push(instance);
    return instance;
  }
};

class JSONModel {
  constructor(data = {}) {
    this.data = data;
    this.bindings = new Set();
  }
  getData() {
    return this.data;
  }
  setData(data) {
    this.data = data;
    this.bindings.forEach((binding) => binding.checkUpdate());
  }
  getProperty(path) {
    return path
      .split("/")
      .filter(Boolean)
      .reduce((value, key) => value?.[key], this.data);
  }
  setProperty(path, value) {
    const keys = path.split("/").filter(Boolean);
    const parent = keys.slice(0, -1).reduce((object, key) => object[key], this.data);
    parent[keys.at(-1)] = value;
    this.bindings.forEach((binding) => binding.checkUpdate());
    return true;
  }
  bindProperty(path) {
    const model = this;
    const binding = {
      value: model.getProperty(path),
      handlers: [],
      destroyed: false,
      attachChange(callback, context) {
        this.handlers.push({ callback, context });
      },
      detachChange(callback, context) {
        this.handlers = this.handlers.filter((entry) => entry.callback !== callback || entry.context !== context);
      },
      checkUpdate() {
        const value = model.getProperty(path);
        if (!Object.is(value, this.value)) {
          this.value = value;
          this.handlers.forEach(({ callback, context }) => callback.call(context));
        }
      },
      destroy() {
        this.destroyed = true;
        model.bindings.delete(this);
      }
    };
    this.bindings.add(binding);
    return binding;
  }
}

class FakeFilter {
  constructor(info, operator, value1, value2) {
    this.info = typeof info === "string" ? { path: info, operator, value1, value2 } : info;
  }
  getFilters() {
    return this.info.filters;
  }
  getTest() {
    return this.info.test;
  }
  getPath() {
    return this.info.path;
  }
  getOperator() {
    return this.info.operator;
  }
  getValue1() {
    return this.info.value1;
  }
  isAnd() {
    return this.info.and ?? false;
  }
}

class FakeWebSocket {
  static OPEN = 1;
  static CLOSED = 3;

  constructor(url) {
    this.url = url;
    this.readyState = 0;
    this.sent = [];
    this.closeCalls = 0;
    this.onopen = null;
    this.onclose = null;
    this.onmessage = null;
    this.onerror = null;
  }
  send(data) {
    this.sent.push(JSON.parse(data));
  }
  close() {
    this.closeCalls++;
    this.readyState = FakeWebSocket.CLOSED;
  }
}

class Controller {
  constructor() {
    this.feedback = [];
    this.busy = [];
    this.routes = new Map();
    this.routeHandlers = new Map();
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
      getModel: (name) => this.models.get(name),
      getResourceBundle: () => ({ getText: (key) => key }),
      getRouter: () => ({
        getRoute: (name) => ({
          attachMatched: (callback, context) => {
            this.routeHandlers.set(name, { callback, context });
            this.routes.set(name, callback.bind(context));
          },
          detachMatched: (callback, context) => {
            const registered = this.routeHandlers.get(name);
            if (registered?.callback === callback && registered.context === context) {
              this.routeHandlers.delete(name);
              this.routes.delete(name);
            }
          },
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
    if (id === "jquery") return fakeJQuery;
    if (id === "sap/m/MessageToast") return { show: (message) => toasts.push(message) };
    if (id === "sap/m/GroupHeaderListItem" || id === "sap/m/Dialog") return class {};
    if (id === "sap/ui/model/Filter") return FakeFilter;
    if (id === "sap/m/MessageBox") {
      return { Action: { OK: "OK", CANCEL: "CANCEL" }, confirm: (message, options) => confirmations.push(options) };
    }
    if (id === "sap/ui/model/FilterOperator") return { Contains: "Contains", EQ: "EQ" };
    if (id === "sap/ui/model/Sorter") return class {};
    if (id === "sap/base/Log") return { debug() {}, error: (...args) => errors.push(args) };
    if (id === "leaflet") return fakeLeaflet;
    if (id === "sap/ui/core/Fragment") return {};
    if (id === "sap/ui/core/library") return { SortOrder: {} };
    if (id === "./Base.controller") return loadController("Base");
    if (id === "./BaseTable.controller") {
      return class extends loadController("Base") {
        init(table) {
          this.table = table;
          this.filters = [];
          this.searchFilters = [];
        }
        sortTable() {}
        setSearchFilters(filters) {
          loadController("BaseTable").prototype.setSearchFilters.call(this, filters);
        }
        applyFilters() {
          loadController("BaseTable").prototype.applyFilters.call(this);
        }
        onExit() {
          this.baseExitCalled = true;
        }
      };
    }
    // Detail-controller navigation constants; no table controller is executed.
    if (id === "./HeatsTable.controller") return { HEAT_MODEL: "heat", HEAT_NAV_MODEL: "heatNav" };
    if (id === "./RacesTable.controller") return { RACE_MODEL: "race", RACE_NAV_MODEL: "raceNav" };
    if (id === "../model/Formatter") return {};
    if (id === "../model/Routes") return routesModule;
    if (id === "sap/ui/core/routing/History") return {};
    throw new Error(`Unmocked controller dependency: ${id}`);
  };
  vm.runInNewContext(
    outputText,
    {
      module,
      exports: module.exports,
      require,
      WebSocket: FakeWebSocket,
      location: { protocol: "https:", host: "test.invalid" },
      removeEventListener() {},
      console: { error: (...args) => errors.push(args), debug() {}, warn() {} }
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

function mapFixture(render = true) {
  leafletState.maps.length = 0;
  const controller = new (loadController("Map"))();
  controller.component.getActiveRegatta = () => Promise.resolve(new JSONModel({ id: 7 }));
  controller.buildClubPopup = (club) => ({ clubId: club.id });
  controller.onInit();
  const container = {};
  const rendered = (element = container) =>
    controller.onMapAfterRendering({ getSource: () => ({ getDomRef: () => element }) });
  const match = (clubId) =>
    controller.routes.get(clubId === undefined ? Routes.Map : Routes.MapClub)({
      getParameter: () => (clubId === undefined ? {} : { clubId: String(clubId) })
    });
  if (render) rendered();
  return { controller, container, rendered, match };
}

test("map renders fixed regatta locations even when the first club request fails", async () => {
  const { controller, container, match } = mapFixture();
  assert.ok(Array.isArray(controller.participatingClubsModel.getData()));
  const instance = controller.map;
  controller.participatingClubsModel.loadData = () =>
    Promise.reject({ statusCode: 503, statusText: "Synthetic club failure" });
  match();
  await setImmediate();
  assert.equal(controller.map, instance);
  assert.equal(instance.container, container);
  assert.equal(instance.options.layers[1].layers.length, 5);
  assert.equal(instance.fitted.at(-1), controller.regattaBounds);
  assert.equal(controller.clubMarkers.size, 0);
  assert.equal(leafletState.maps.length, 1);
});

test("map renders fixed locations when active-regatta lookup fails", async () => {
  const { controller, match } = mapFixture();
  controller.component.getActiveRegatta = () => Promise.reject(new Error("Synthetic regatta failure"));
  match();
  await setImmediate();
  assert.ok(controller.map);
  assert.equal(controller.map.options.layers[1].layers.length, 5);
  assert.equal(controller.map.fitted.at(-1), controller.regattaBounds);
});

test("a retry refreshes club markers, bounds, and focus without replacing the map or overlay", async () => {
  const { controller, match } = mapFixture();
  const instance = controller.map;
  const overlay = controller.clubsLayer;
  controller.participatingClubsModel.loadData = () => Promise.reject({ statusText: "Unavailable" });
  match(1);
  await setImmediate();

  let clubs = [{ id: 1, latitude: 49, longitude: 8 }];
  controller.participatingClubsModel.loadData = async (url) => {
    assert.equal(url, "/api/regattas/7/clubs");
    controller.participatingClubsModel.setData(clubs);
  };
  match(1);
  await setImmediate();
  assert.equal(controller.map, instance);
  assert.equal(controller.clubsLayer, overlay);
  assert.equal(overlay.layers.length, 1);
  assert.equal(instance.focused.at(-1).position.lat, 49);
  const oldMarker = controller.clubMarkers.get(1);

  clubs = [
    { id: 1, latitude: 50, longitude: 9 },
    { id: 2, latitude: 0, longitude: 0 }
  ];
  match(2);
  await setImmediate();
  assert.equal(overlay.layers.length, 2);
  assert.notEqual(controller.clubMarkers.get(1), oldMarker);
  assert.equal(controller.clubMarkers.get(1).position.lat, 50);
  assert.equal(instance.focused.at(-1).position.lat, 0);
  assert.equal(instance.focused.at(-1).position.lng, 0);

  clubs = [{ id: 2, latitude: 0, longitude: 0 }];
  match();
  await setImmediate();
  assert.equal(controller.clubMarkers.has(1), false);
  assert.equal(overlay.layers.length, 1);
  assert.equal(instance.fitted.at(-1), controller.clubBounds);
  assert.equal(leafletState.maps.length, 1);
});

test("club data arriving before HTML rendering is used without premature map creation", async () => {
  const { controller, rendered, match } = mapFixture(false);
  controller.participatingClubsModel.loadData = async () => {
    controller.participatingClubsModel.setData([{ id: 1, latitude: 49, longitude: 8 }]);
  };
  match(1);
  await setImmediate();
  assert.equal(leafletState.maps.length, 0);
  assert.equal(controller.map, undefined);
  rendered();
  assert.equal(controller.clubMarkers.size, 1);
  assert.equal(controller.map.focused.at(-1).position.lat, 49);
});

test("backend decimal-string coordinates populate club markers before rendering and on later refresh", async () => {
  for (const renderFirst of [false, true]) {
    const { controller, rendered, match } = mapFixture(renderFirst);
    controller.participatingClubsModel.loadData = async () => {
      controller.participatingClubsModel.setData([
        { id: 1, latitude: "49.412944", longitude: "8.690510" },
        { id: 2, latitude: "0.000000", longitude: "-8.500000" }
      ]);
    };
    match(1);
    await setImmediate();
    if (!renderFirst) rendered();
    assert.equal(controller.clubsLayer.layers.length, 2);
    assert.equal(controller.clubMarkers.get(1).position.lat, 49.412944);
    assert.equal(controller.clubMarkers.get(1).position.lng, 8.69051);
    assert.equal(controller.clubMarkers.get(2).position.lat, 0);
    assert.equal(controller.clubMarkers.get(2).position.lng, -8.5);
    assert.equal(controller.map.focused.at(-1).position.lat, 49.412944);
    match();
    await setImmediate();
    assert.equal(controller.map.fitted.at(-1), controller.clubBounds);
    assert.equal(controller.clubBounds.points[0].lat, 49.412944);
    assert.equal(leafletState.maps.length, 1);
  }
});

test("malformed club responses and invalid coordinates preserve the static map fallback", async () => {
  const { controller, match } = mapFixture();
  let response = { clubs: [] };
  controller.participatingClubsModel.loadData = async () => controller.participatingClubsModel.setData(response);
  match();
  await setImmediate();
  assert.equal(controller.clubsLayer.layers.length, 0);
  assert.equal(controller.map.fitted.at(-1), controller.regattaBounds);
  response = [null, {}, { id: null, latitude: 49, longitude: 8 }];
  for (const invalid of [undefined, null, "", "   ", true, false, [], {}, "invalid", "NaN", "Infinity", Infinity]) {
    response.push({ id: 1, latitude: invalid, longitude: 8 }, { id: 2, latitude: 49, longitude: invalid });
  }
  response.push(
    { id: 3, latitude: 91, longitude: 0 },
    { id: 4, latitude: "-91", longitude: "0" },
    { id: 5, latitude: "49", longitude: "181" },
    { id: 6, latitude: 49, longitude: -181 }
  );
  match();
  await setImmediate();
  assert.equal(controller.clubMarkers.size, 0);
  assert.equal(controller.map.fitted.at(-1), controller.regattaBounds);
});

test("map rerenders preserve the current container or release a replaced one", () => {
  const { controller, rendered } = mapFixture();
  const first = controller.map;
  rendered();
  assert.equal(controller.map, first);
  controller.onAfterShow();
  assert.equal(first.resized, 1);
  rendered({});
  assert.equal(first.removed, 1);
  assert.notEqual(controller.map, first);
  controller.onExit();
  assert.equal(leafletState.maps.at(-1).removed, 1);
  assert.equal(controller.map, undefined);
  assert.equal(controller.routes.has(Routes.Map), false);
  assert.equal(controller.routes.has(Routes.MapClub), false);
  controller.onExit();
  assert.equal(leafletState.maps.at(-1).removed, 1);
});

test("map uses the latest route focus when club loading completes and ignores completion after destruction", async () => {
  const { controller, match } = mapFixture();
  let resolve;
  controller.component.getActiveRegatta = () =>
    new Promise((done) => {
      resolve = done;
    });
  // Share one pending bootstrap lookup as the real component does.
  const pending = controller.component.getActiveRegatta();
  controller.component.getActiveRegatta = () => pending;
  controller.participatingClubsModel.loadData = async () => {
    controller.participatingClubsModel.setData([
      { id: 1, latitude: 49, longitude: 8 },
      { id: 2, latitude: 50, longitude: 9 }
    ]);
  };
  match(1);
  match(2);
  resolve(new JSONModel({ id: 7 }));
  await setImmediate();
  assert.ok(controller.map.focused.every((focus) => focus.position.lat === 50));
  const instance = controller.map;
  let finish;
  controller.participatingClubsModel.loadData = () =>
    new Promise((done) => {
      finish = done;
    });
  match(1);
  await setImmediate();
  controller.onExit();
  controller.view.isDestroyed = () => true;
  finish();
  await setImmediate();
  assert.equal(controller.map, undefined);
  assert.equal(instance.removed, 1);
  assert.equal(leafletState.maps.length, 1);
});

function loadFormatter() {
  const types = {};
  vm.runInNewContext(
    ts.transpileModule(readFileSync(new URL("../webapp/model/Types.ts", import.meta.url), "utf8"), {
      compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 }
    }).outputText,
    { exports: types }
  );
  const module = { exports: {} };
  const { outputText } = ts.transpileModule(
    readFileSync(new URL("../webapp/model/Formatter.ts", import.meta.url), "utf8"),
    { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022, esModuleInterop: true } }
  );
  vm.runInNewContext(outputText, {
    exports: module.exports,
    module,
    require: (id) => {
      if (id === "./Types") return types;
      if (id === "sap/base/Log") return { warning() {} };
      if (id === "sap/ui/core/library") return { IndicationColor: {} };
      if (id === "sap/m/library") return { Priority: {} };
      throw new Error(`Unmocked formatter dependency: ${id}`);
    }
  });
  return module.exports.default;
}

test("schedule weekday and UTC clock remain consistent across browser timezones and DST boundaries", () => {
  const formatter = loadFormatter();
  const original = process.env.TZ;
  try {
    for (const timezone of ["UTC", "Europe/Berlin", "America/Los_Angeles", "Pacific/Auckland"]) {
      process.env.TZ = timezone;
      assert.equal(formatter.dayTimeIsoLabel("2026-05-22T23:30:00Z"), "Fr, 23:30", timezone);
      assert.equal(formatter.dayTimeIsoLabel("2026-05-23T00:30:00Z"), "Sa, 00:30", timezone);
      assert.equal(formatter.dayTimeIsoLabel("2026-05-23T00:30:00+02:00"), "Fr, 22:30", timezone);
      assert.equal(formatter.dayTimeIsoLabel("2026-03-29T00:30:00Z"), "So, 00:30", timezone);
      assert.equal(formatter.dayTimeIsoLabel("2026-03-29T01:30:00Z"), "So, 01:30", timezone);
      assert.equal(formatter.weekDayDateLabel("2026-05-22"), "Fr, 22.05.2026", timezone);
    }
    assert.equal(formatter.dayTimeIsoLabel(undefined), "");
    assert.equal(formatter.weekDayDateLabel(undefined), "");
  } finally {
    if (original === undefined) delete process.env.TZ;
    else process.env.TZ = original;
  }
});

test("UTC schedule formatting does not alter local notification and timekeeping timestamp formats", () => {
  const formatter = loadFormatter();
  const original = process.env.TZ;
  try {
    process.env.TZ = "Europe/Berlin";
    assert.equal(formatter.dateTime("2026-05-22T23:30:00Z"), "23.05.2026 01:30");
    assert.equal(formatter.timestamp("2026-05-22T23:30:00Z"), "23.05.2026 01:30:00.000");
  } finally {
    if (original === undefined) delete process.env.TZ;
    else process.env.TZ = original;
  }
});

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
  assert.equal(controller.routes.get(Routes.Statistics)({}), undefined);
  await setImmediate();
  assert.deepEqual(controller.feedback, [false]);
  controller.feedback.length = 0;
  assert.equal(controller.onRefreshButtonPress({}), undefined);
  assert.deepEqual(controller.busy, [true]);
  await setImmediate();
  assert.deepEqual(controller.busy, [true, false]);
  assert.deepEqual(controller.feedback, [false]);
});

test("notifications load on every route match, not directly during initialization", async () => {
  const controller = new (loadController("Notifications"))();
  controller.control.setEscapeHandler = () => {};
  let loads = 0;
  controller.loadNotifications = () => {
    loads++;
    return Promise.resolve();
  };

  assert.equal(controller.onInit(), undefined);
  assert.equal(loads, 0, "initialization must not load notifications before the route matches");
  assert.ok(controller.models.has("notifications"));
  assert.ok(controller.models.has("dialog"));

  const onMatched = controller.routes.get(Routes.Notifications);
  assert.equal(typeof onMatched, "function");
  assert.equal(onMatched({}), undefined);
  assert.equal(loads, 1, "the first navigation must load notifications");
  await setImmediate();

  assert.equal(onMatched({}), undefined);
  assert.equal(loads, 2, "revisiting the same controller must reload notifications");
  await setImmediate();
  assert.deepEqual(controller.feedback, []);

  assert.equal(controller.onRefresh(), undefined);
  assert.equal(loads, 3, "manual refresh must still load notifications");
});

function notificationsFixture() {
  ajaxRequests.length = 0;
  toasts.length = 0;
  confirmations.length = 0;
  const controller = new (loadController("Notifications"))();
  const identity = new JSONModel({ authenticated: true, username: "admin", scope: "admin" });
  controller.models.set("identity", identity);
  controller.component.getActiveRegatta = () => Promise.resolve(new JSONModel({ id: 7 }));
  const dialog = {
    opened: 0,
    closed: 0,
    visible: false,
    isOpen() {
      return this.visible;
    },
    setEscapeHandler(handler) {
      this.escapeHandler = handler;
    },
    escape() {
      this.escapeHandler({ resolve: () => this.close(), reject() {} });
    },
    open() {
      this.opened++;
      this.visible = true;
    },
    close() {
      this.closed++;
      this.visible = false;
    }
  };
  controller.control = dialog;
  controller.onInit();
  const eventFor = (notification, parameters = {}) => ({
    getSource: () => ({ getBindingContext: () => ({ getObject: () => notification }) }),
    getParameters: () => parameters
  });
  const requests = (method) => ajaxRequests.filter((request) => request.type === method);
  const form = controller.models.get("dialog");
  const state = controller.models.get("notificationState");
  const list = controller.models.get("notifications");
  const add = (title = "New notification") => {
    const previousOpens = dialog.opened;
    controller.onAddNotification();
    if (dialog.opened !== previousOpens) {
      form.setData({ title, text: "Details", priority: "2", visible: true });
    }
  };
  return { controller, identity, dialog, eventFor, requests, form, state, list, add };
}

test("notification loading awaits the GET response, not just AJAX dispatch", async () => {
  const { controller, requests, list } = notificationsFixture();
  let settled = false;
  const pending = controller.loadNotifications().then((result) => {
    settled = true;
    return result;
  });
  await setImmediate();
  assert.equal(settled, false);
  assert.equal(requests("GET").length, 1);
  assert.equal(requests("GET")[0].url, "/api/regattas/7/notifications");
  const notifications = [{ id: 1, title: "Current", visible: false }];
  requests("GET")[0].succeed(notifications);
  assert.equal(await pending, true);
  assert.equal(list.getData(), notifications);
});

test("an older route read cannot overwrite a newer notification refresh", async () => {
  const { controller, requests, list } = notificationsFixture();
  controller.routes.get(Routes.Notifications)({});
  await setImmediate();
  controller.onRefresh();
  await setImmediate();
  const newest = [{ id: 1, title: "Updated by admin" }];
  requests("GET")[1].succeed(newest);
  await setImmediate();
  requests("GET")[0].succeed([{ id: 1, title: "Obsolete" }]);
  await setImmediate();
  assert.equal(list.getData(), newest);
});

test("obsolete notification failures are silent while current failures remain visible and retryable", async () => {
  const { controller, requests, list } = notificationsFixture();
  const old = controller.loadNotifications();
  await setImmediate();
  const current = controller.loadNotifications();
  await setImmediate();
  const newest = [{ id: 1, title: "Newest" }];
  requests("GET")[1].succeed(newest);
  assert.equal(await current, true);
  requests("GET")[0].fail();
  assert.equal(await old, false);
  assert.deepEqual(toasts, []);
  const failed = controller.loadNotifications();
  await setImmediate();
  requests("GET")[2].fail();
  assert.equal(await failed, false);
  assert.deepEqual(toasts, ["notifications.error.loadFailed"]);
  assert.equal(list.getData(), newest);
  const retry = controller.loadNotifications();
  await setImmediate();
  requests("GET")[3].succeed([]);
  assert.equal(await retry, true);
  assert.equal(list.getData().length, 0, "successful empty responses must replace older rows");
});

test("notification read identity covers delayed regatta lookup as well as GET completion", async () => {
  const { controller, requests, list } = notificationsFixture();
  const lookups = [];
  controller.component.getActiveRegatta = () => new Promise((resolve) => lookups.push(resolve));
  const old = controller.loadNotifications();
  const current = controller.loadNotifications();
  lookups[1](new JSONModel({ id: 8 }));
  await setImmediate();
  assert.equal(requests("GET")[0].url, "/api/regattas/8/notifications");
  lookups[0](new JSONModel({ id: 7 }));
  assert.equal(await old, false);
  assert.equal(requests("GET").length, 1, "obsolete lookup must not start another GET");
  const newest = [{ id: 2, title: "Regatta 8" }];
  requests("GET")[0].succeed(newest);
  assert.equal(await current, true);
  assert.equal(list.getData(), newest);
});

test("all notification mutations reconcile with a newer read that obsolete reads cannot replace", async () => {
  for (const kind of ["create", "update", "visibility", "delete"]) {
    const { controller, requests, list } = notificationsFixture();
    const oldRead = controller.loadNotifications();
    await setImmediate();
    const data = { title: "Updated", text: "Text", priority: "3", visible: false };
    const write = controller.writeNotification(
      kind,
      kind === "create" ? undefined : 1,
      kind === "delete" ? undefined : kind === "visibility" ? { visible: false } : data
    );
    await setImmediate();
    const mutation = ajaxRequests.find((request) => request.type !== "GET");
    assert.ok(mutation, kind);
    mutation.succeed({ id: 1, ...data });
    assert.equal(await write, true);
    await setImmediate();
    assert.equal(requests("GET").length, 2, kind);
    const newest = kind === "delete" ? [] : [{ id: 1, ...data }];
    requests("GET")[1].succeed(newest);
    await setImmediate();
    requests("GET")[0].succeed([{ id: 1, title: "Old value" }]);
    assert.equal(await oldRead, false);
    assert.equal(list.getData(), newest, kind);
  }
});

test("starting a notification mutation invalidates older reads before the write completes", async () => {
  const { controller, requests, list } = notificationsFixture();
  const current = [{ id: 1, title: "Current local state" }];
  list.setData(current);
  const oldRead = controller.loadNotifications();
  await setImmediate();
  const write = controller.writeNotification("update", 1, { title: "Edited", priority: 1, visible: true });
  requests("GET")[0].succeed([{ id: 1, title: "Stale read" }]);
  assert.equal(await oldRead, false);
  assert.equal(list.getData(), current);
  requests("PUT")[0].fail();
  assert.equal(await write, false);
});

test("failed visibility writes reconcile the switch state without accepting earlier reads", async () => {
  const { controller, requests, list } = notificationsFixture();
  const oldRead = controller.loadNotifications();
  await setImmediate();
  const write = controller.writeNotification("visibility", 1, { visible: false });
  assert.deepEqual(JSON.parse(requests("PUT")[0].data), { visible: false });
  requests("PUT")[0].fail();
  assert.equal(await write, false);
  await setImmediate();
  requests("GET")[1].succeed([{ id: 1, visible: true }]);
  await setImmediate();
  requests("GET")[0].succeed([{ id: 1, visible: false }]);
  assert.equal(await oldRead, false);
  assert.equal(list.getData()[0].visible, true);
  assert.deepEqual(toasts, ["notifications.error.visibilityFailed"]);
});

test("notification Save blocks duplicate creation during bootstrap and the pending POST", async () => {
  const { controller, requests, dialog, state, add } = notificationsFixture();
  let resolveRegatta;
  controller.component.getActiveRegatta = () => new Promise((resolve) => (resolveRegatta = resolve));
  add();
  assert.equal(controller.onSaveNotification(), undefined);
  controller.onSaveNotification();
  assert.equal(state.getProperty("/saving"), true, "lock before the first await");
  assert.equal(requests("POST").length, 0);
  controller.onCancelDialog();
  dialog.escape();
  controller.onAddNotification();
  assert.equal(dialog.opened, 1);
  assert.equal(dialog.closed, 0);
  resolveRegatta(new JSONModel({ id: 7 }));
  await setImmediate();
  controller.onSaveNotification();
  assert.equal(requests("POST").length, 1);
  assert.deepEqual(JSON.parse(requests("POST")[0].data), {
    title: "New notification",
    text: "Details",
    priority: 2,
    visible: true
  });
  assert.equal(requests("POST")[0].url, "/api/regattas/7/notifications");
  assert.equal(dialog.closed, 0);
  controller.component.getActiveRegatta = () => Promise.resolve(new JSONModel({ id: 7 }));
  requests("POST")[0].succeed({ id: 1 });
  await setImmediate();
  assert.equal(state.getProperty("/saving"), false);
  assert.equal(dialog.closed, 1);
  assert.equal(requests("GET").length, 1);
  assert.equal(requests("GET")[0].settled, false, "list reload must not hold the write lock");
});

test("failed notification saves retain the form and allow a deliberate retry", async () => {
  const { controller, requests, form, state, dialog, add } = notificationsFixture();
  add("Recoverable title");
  const input = form.getData();
  controller.onSaveNotification();
  await setImmediate();
  requests("POST")[0].fail();
  await setImmediate();
  assert.equal(state.getProperty("/saving"), false);
  assert.equal(form.getData(), input);
  assert.equal(dialog.isOpen(), true);
  assert.deepEqual(toasts, ["notifications.error.createFailed"]);
  controller.onSaveNotification();
  await setImmediate();
  assert.equal(requests("POST").length, 2);
  requests("POST")[1].succeed({ id: 1 });
  await setImmediate();
  assert.equal(dialog.closed, 1);
  assert.equal(state.getProperty("/saving"), false);
});

test("regatta bootstrap failure unlocks notification Save without discarding the form", async () => {
  const { controller, requests, form, state, dialog, add } = notificationsFixture();
  add();
  controller.component.getActiveRegatta = () => Promise.reject(new Error("Synthetic bootstrap failure"));
  controller.onSaveNotification();
  await setImmediate();
  assert.equal(requests("POST").length, 0);
  assert.equal(state.getProperty("/saving"), false);
  assert.equal(form.getProperty("/title"), "New notification");
  assert.equal(dialog.isOpen(), true);
  assert.deepEqual(toasts, ["notifications.error.regattaFailed"]);
});

test("notification edits are single-flight and use a snapshot of the originating form", async () => {
  const { controller, eventFor, requests, form, state, dialog } = notificationsFixture();
  controller.onEditNotification(eventFor({ id: 7, title: "Original", text: "Text", priority: "1", visible: false }));
  const input = form.getData();
  controller.onSaveNotification();
  input.title = "Modified after submission";
  controller.onSaveNotification();
  assert.equal(requests("PUT").length, 1);
  assert.equal(JSON.parse(requests("PUT")[0].data).title, "Original");
  assert.equal(requests("PUT")[0].url, "/api/notifications/7");
  requests("PUT")[0].fail();
  await setImmediate();
  assert.equal(dialog.isOpen(), true);
  assert.equal(state.getProperty("/saving"), false);
  assert.equal(form.getData(), input);
  assert.deepEqual(toasts, ["notifications.error.updateFailed"]);
  controller.onSaveNotification();
  controller.onSaveNotification();
  assert.equal(requests("PUT").length, 2);
  requests("PUT")[1].succeed({ id: 7 });
  await setImmediate();
  assert.equal(dialog.closed, 1);
  assert.equal(state.getProperty("/saving"), false);
});

test("pending notification saves block Cancel, Escape, Add, and Edit until the write settles", async () => {
  for (const edit of [false, true]) {
    for (const succeed of [false, true]) {
      const { controller, eventFor, requests, form, state, dialog, add } = notificationsFixture();
      if (edit) controller.onEditNotification(eventFor({ id: 1, title: "Old", priority: 1, visible: true }));
      else add("Old");
      const originalForm = form.getData();
      controller.onSaveNotification();
      await setImmediate();
      controller.onCancelDialog();
      dialog.escape();
      add("New form");
      controller.onEditNotification(eventFor({ id: 2, title: "Another edit", priority: 2, visible: false }));
      assert.equal(dialog.isOpen(), true);
      assert.equal(dialog.closed, 0);
      assert.equal(dialog.opened, 1);
      assert.equal(form.getData(), originalForm);
      controller.onSaveNotification();
      assert.equal(requests(edit ? "PUT" : "POST").length, 1);
      const request = requests(edit ? "PUT" : "POST")[0];
      if (succeed) request.succeed({ id: 1 });
      else request.fail();
      await setImmediate();
      assert.equal(state.getProperty("/saving"), false);
      assert.equal(dialog.isOpen(), !succeed);
      assert.equal(dialog.closed, succeed ? 1 : 0);
      if (succeed) assert.deepEqual(Object.keys(form.getData()), []);
      else assert.equal(form.getData(), originalForm);
      add("New form");
      assert.equal(dialog.isOpen(), true);
      assert.equal(form.getProperty("/title"), "New form");
    }
  }
});

test("inline priority updates cannot close an unrelated notification dialog", async () => {
  const { controller, eventFor, requests, form, dialog, add } = notificationsFixture();
  add("Unrelated form");
  controller.onPriorityChange(
    eventFor(
      { id: 1, title: "Existing", text: "Text", priority: 1, visible: true },
      { selectedItem: { getKey: () => "3" } }
    )
  );
  assert.equal(JSON.parse(requests("PUT")[0].data).priority, 3);
  requests("PUT")[0].succeed({ id: 1 });
  await setImmediate();
  assert.equal(dialog.isOpen(), true);
  assert.equal(dialog.closed, 0);
  assert.equal(form.getProperty("/title"), "Unrelated form");
});

test("session changes clear administrative notifications and reject old reads even after the same user logs in again", async () => {
  const { controller, identity, requests, list } = notificationsFixture();
  list.setData([{ id: 1, title: "Private" }]);
  const pending = controller.loadNotifications();
  await setImmediate();
  identity.setProperty("/authenticated", false);
  identity.setProperty("/authenticated", true);
  assert.equal(list.getData().length, 0);
  requests("GET")[0].succeed([{ id: 1, title: "Old session" }]);
  assert.equal(await pending, false);
  assert.equal(list.getData().length, 0);
  assert.deepEqual(toasts, []);
  const current = controller.loadNotifications();
  await setImmediate();
  requests("GET")[1].succeed([{ id: 2, title: "Current session" }]);
  assert.equal(await current, true);
  assert.equal(list.getData()[0].id, 2);
});

test("session changes close the locked form while old writes remain silent", async () => {
  const { controller, identity, requests, form, dialog, state, add } = notificationsFixture();
  add("Old session");
  controller.onSaveNotification();
  await setImmediate();
  identity.setProperty("/username", "another-admin");
  add("Current session");
  assert.equal(dialog.isOpen(), false, "a new form stays blocked until the pending write settles");
  assert.deepEqual(Object.keys(form.getData()), []);
  requests("POST")[0].succeed({ id: 1 });
  await setImmediate();
  assert.equal(dialog.isOpen(), false);
  assert.equal(dialog.closed, 1);
  assert.equal(state.getProperty("/saving"), false);
  assert.equal(requests("GET").length, 0);
  assert.deepEqual(toasts, []);
  add("Current session");
  assert.equal(dialog.isOpen(), true);
  assert.equal(form.getProperty("/title"), "Current session");
});

test("notification deletion waits for confirmation and ignores confirmations from an obsolete session", async () => {
  const { controller, identity, eventFor, requests } = notificationsFixture();
  controller.onDeleteNotification(eventFor({ id: 1 }));
  assert.equal(requests("DELETE").length, 0);
  confirmations[0].onClose("CANCEL");
  assert.equal(requests("DELETE").length, 0);
  controller.onDeleteNotification(eventFor({ id: 1 }));
  confirmations[1].onClose("OK");
  assert.equal(requests("DELETE").length, 1);
  assert.equal(requests("DELETE")[0].url, "/api/notifications/1");
  requests("DELETE")[0].succeed();
  await setImmediate();
  assert.equal(requests("GET").length, 1);
  controller.onDeleteNotification(eventFor({ id: 2 }));
  identity.setProperty("/scope", "user");
  confirmations[2].onClose("OK");
  assert.equal(requests("DELETE").length, 1);
});

test("destroying notification views invalidates reads and writes and detaches all identity and route listeners", async () => {
  const { controller, identity, requests, state, list, dialog, add } = notificationsFixture();
  const pending = controller.loadNotifications();
  await setImmediate();
  add();
  controller.onSaveNotification();
  await setImmediate();
  const bindings = [...identity.bindings];
  const currentData = list.getData();
  controller.onExit();
  state.setProperty = () => assert.fail("late write must not update destroyed state");
  requests("POST")[0].succeed({ id: 1 });
  requests("GET")[0].succeed([{ id: 1, title: "Late response" }]);
  assert.equal(await pending, false);
  await setImmediate();
  assert.equal(list.getData(), currentData);
  assert.equal(dialog.closed, 0);
  assert.equal(requests("GET").length, 1);
  assert.deepEqual(toasts, []);
  assert.equal(identity.bindings.size, 0);
  assert.ok(bindings.every((binding) => binding.destroyed && binding.handlers.length === 0));
  assert.equal(controller.routes.has(Routes.Notifications), false);
  controller.onExit();
});

test("closing a notification controller during regatta lookup prevents later HTTP dispatch", async () => {
  const { controller, requests, add } = notificationsFixture();
  let resolveRegatta;
  const bootstrap = new Promise((resolve) => (resolveRegatta = resolve));
  controller.component.getActiveRegatta = () => bootstrap;
  const read = controller.loadNotifications();
  add();
  controller.onSaveNotification();
  controller.onExit();
  resolveRegatta(new JSONModel({ id: 7 }));
  assert.equal(await read, false);
  await setImmediate();
  assert.equal(requests("GET").length, 0);
  assert.equal(requests("POST").length, 0);
  assert.deepEqual(toasts, []);
});

test("notification Save, Cancel, Add, and Edit bind their enabled state to the pending write", () => {
  const xml = readFileSync(new URL("../webapp/view/Notifications.view.xml", import.meta.url), "utf8");
  const buttons = Array.from(xml.matchAll(/<Button\b[\s\S]*?\/>/g), (match) => match[0]);
  for (const handler of ["onSaveNotification", "onCancelDialog", "onAddNotification", "onEditNotification"]) {
    const button = buttons.find((item) => item.includes(`press=".${handler}"`));
    assert.ok(button, handler);
    assert.ok(button.includes('enabled="{=!${notificationState>/saving}}"'), handler);
  }
  const save = buttons.find((item) => item.includes('press=".onSaveNotification"'));
  assert.ok(save.includes('busy="{notificationState>/saving}"'));
});

test("Cancel and Escape work when no save is pending and clear the closed form", () => {
  const { controller, dialog, form, add } = notificationsFixture();
  add();
  controller.onCancelDialog();
  assert.equal(dialog.isOpen(), false);
  assert.deepEqual(Object.keys(form.getData()), []);
  add();
  dialog.escape();
  assert.equal(dialog.isOpen(), false);
  assert.deepEqual(Object.keys(form.getData()), []);
});

test("a blank notification title is still validated before locking the form", async () => {
  const { controller, dialog, state, requests, add } = notificationsFixture();
  add("");
  controller.onSaveNotification();
  await setImmediate();
  assert.equal(dialog.isOpen(), true);
  assert.equal(state.getProperty("/saving"), false);
  assert.equal(requests("POST").length, 0);
  assert.deepEqual(toasts, ["notifications.validation.titleRequired"]);
});

test("queued Save events during the closing animation cannot create another notification", async () => {
  const { controller, dialog, form, requests, add } = notificationsFixture();
  add();
  controller.onSaveNotification();
  await setImmediate();
  requests("POST")[0].succeed({ id: 1 });
  await setImmediate();
  assert.deepEqual(Object.keys(form.getData()), []);
  // UI5 reports isOpen() throughout its closing animation.
  dialog.visible = true;
  controller.onSaveNotification();
  await setImmediate();
  assert.equal(requests("POST").length, 1);
  assert.deepEqual(toasts, ["notifications.success.created"]);
});

function timekeepingFixture() {
  const controller = new (loadController("Timekeeping"))();
  const icons = [];
  controller.control.setIcon = (icon) => icons.push(icon);
  controller.onInit();
  for (const name of ["timestrip", "aquariusHeats", "heats", "bibs"]) controller.models.get(name).setData([]);
  controller.routes.get(Routes.Timekeeping)({});
  return { controller, icons };
}

test("timekeeping reconnect invalidates and detaches the previous socket before closing it", () => {
  const { controller } = timekeepingFixture();
  const previous = controller.socket;
  const close = previous.close.bind(previous);
  previous.close = () => {
    assert.equal(controller.socket, undefined, "invalidate ownership before close can dispatch any events");
    for (const handler of ["onopen", "onclose", "onmessage", "onerror"]) assert.equal(previous[handler], null);
    close();
  };
  controller.onStatusButtonPress();
  assert.notEqual(controller.socket, previous);
  assert.equal(previous.closeCalls, 1);
});

test("queued old timekeeping callbacks cannot affect the replacement connection", () => {
  const { controller, icons } = timekeepingFixture();
  const previous = controller.socket;
  const queued = { open: previous.onopen, close: previous.onclose, message: previous.onmessage };
  controller.onStatusButtonPress();
  const current = controller.socket;
  current.readyState = FakeWebSocket.OPEN;
  current.onopen({});
  const currentData = [{ time: "current" }];
  controller.models.get("timestrip").setData(currentData);
  const sent = current.sent.length;
  const iconUpdates = icons.length;

  queued.open({});
  queued.close({});
  queued.message({ data: JSON.stringify({ TimeStrip: { time_stamps: [{ time: "obsolete" }] } }) });
  assert.doesNotThrow(() => queued.message({ data: "obsolete malformed JSON" }));

  assert.equal(controller.socket, current);
  assert.equal(controller.models.get("timestrip").getData(), currentData);
  assert.equal(current.sent.length, sent, "an old open callback must not send another request on the new socket");
  assert.equal(icons.length, iconUpdates);
  assert.equal(icons.at(-1), "sap-icon://connected");
});

test("hiding timekeeping closes its socket and makes previously queued callbacks harmless", () => {
  const { controller, icons } = timekeepingFixture();
  const socket = controller.socket;
  const queued = { open: socket.onopen, close: socket.onclose, message: socket.onmessage };
  const currentData = controller.models.get("timestrip").getData();
  controller.onBeforeHide();
  const iconUpdates = icons.length;
  assert.equal(controller.socket, undefined);
  assert.equal(socket.closeCalls, 1);
  queued.open({});
  queued.close({});
  queued.message({ data: JSON.stringify({ TimeStrip: { time_stamps: [{ time: "obsolete" }] } }) });
  assert.equal(controller.models.get("timestrip").getData(), currentData);
  assert.equal(icons.length, iconUpdates);
  controller.disconnect();
  assert.equal(socket.closeCalls, 1, "disconnect must be idempotent");
});

test("destroying timekeeping closes its socket and retains inherited cleanup", () => {
  const { controller } = timekeepingFixture();
  const socket = controller.socket;
  assert.equal(controller.onExit(), undefined);
  assert.equal(controller.socket, undefined);
  assert.equal(socket.closeCalls, 1);
  for (const handler of ["onopen", "onclose", "onmessage", "onerror"]) assert.equal(socket[handler], null);
  assert.equal(controller.baseExitCalled, true);
});

test("the active timekeeping socket still requests and updates data normally", () => {
  const { controller, icons } = timekeepingFixture();
  const socket = controller.socket;
  assert.equal(socket.url, "wss://test.invalid/api/timekeeping");
  socket.readyState = FakeWebSocket.OPEN;
  socket.onopen({});
  assert.deepEqual(socket.sent, [{ GetTimestrip: null }]);
  assert.equal(icons.at(-1), "sap-icon://connected");

  socket.onmessage({ data: JSON.stringify({ TimeStrip: { time_stamps: [{ time: "existing" }] } }) });
  socket.onmessage({ data: JSON.stringify({ Timestamp: { timestamp: { time: "added" } } }) });
  assert.deepEqual(
    Array.from(controller.models.get("timestrip").getData(), (timestamp) => timestamp.time),
    ["added", "existing"]
  );
  socket.onmessage({ data: JSON.stringify({ AquariusHeats: { heats: [{ id: 7 }] } }) });
  assert.equal(controller.models.get("aquariusHeats").getData()[0].id, 7);
  socket.onmessage({ data: JSON.stringify({ HeatsReadyToStart: { heats: [{ id: 8 }] } }) });
  assert.equal(controller.models.get("heats").getData()[0].id, 8);

  socket.readyState = FakeWebSocket.CLOSED;
  socket.onclose({});
  assert.equal(controller.socket, undefined);
  assert.equal(icons.at(-1), "sap-icon://disconnected");
});

function monitoringFixture() {
  const controller = new (loadController("Monitoring"))();
  const icons = [];
  const enabled = [];
  controller.control.setIcon = (icon) => icons.push(icon);
  controller.control.setEnabled = (value) => enabled.push(value);
  controller.onInit();
  controller.routes.get(Routes.Monitoring)({});
  return { controller, icons, enabled };
}

test("monitoring reconnect invalidates and detaches the previous socket before closing it", () => {
  const { controller } = monitoringFixture();
  const previous = controller.socket;
  previous.readyState = FakeWebSocket.OPEN;
  previous.onopen({});
  const close = previous.close.bind(previous);
  previous.close = () => {
    assert.equal(controller.socket, undefined);
    for (const handler of ["onopen", "onclose", "onmessage", "onerror"]) assert.equal(previous[handler], null);
    close();
  };
  controller.onStatusButtonPress();
  assert.notEqual(controller.socket, previous);
  assert.equal(previous.closeCalls, 1);
});

test("old monitoring callbacks cannot alter metrics or re-enable a replacement connection's button", () => {
  const { controller, icons, enabled } = monitoringFixture();
  const previous = controller.socket;
  previous.readyState = FakeWebSocket.OPEN;
  previous.onopen({});
  const queued = { open: previous.onopen, close: previous.onclose, message: previous.onmessage };
  const currentData = [{ name: "current", value: 1, group: "7" }];
  controller.models.get("monitoring").setData(currentData);
  controller.onStatusButtonPress();
  const current = controller.socket;
  const iconUpdates = icons.length;
  const enabledUpdates = enabled.length;

  queued.open({});
  queued.close({});
  queued.message({ data: JSON.stringify({ Update: { monitoring: { sys: { uptime: { secs: 99 } } } } }) });
  assert.doesNotThrow(() => queued.message({ data: "obsolete malformed JSON" }));
  assert.equal(controller.socket, current);
  assert.equal(controller.models.get("monitoring").getData(), currentData);
  assert.equal(icons.length, iconUpdates);
  assert.equal(enabled.length, enabledUpdates);
  assert.equal(enabled.at(-1), false);
  assert.equal(icons.at(-1), "sap-icon://disconnected");

  current.readyState = FakeWebSocket.OPEN;
  current.onopen({});
  queued.close({});
  assert.equal(enabled.at(-1), true);
  assert.equal(icons.at(-1), "sap-icon://connected");
});

test("hiding monitoring closes its socket and makes previously queued callbacks harmless", () => {
  const { controller, icons, enabled } = monitoringFixture();
  const socket = controller.socket;
  const queued = { open: socket.onopen, close: socket.onclose, message: socket.onmessage };
  const currentData = controller.models.get("monitoring").getData();
  controller.onBeforeHide();
  const iconUpdates = icons.length;
  const enabledUpdates = enabled.length;
  assert.equal(controller.socket, undefined);
  assert.equal(socket.closeCalls, 1);
  queued.open({});
  queued.close({});
  queued.message({ data: JSON.stringify({ Update: { monitoring: { sys: { uptime: { secs: 99 } } } } }) });
  assert.equal(controller.models.get("monitoring").getData(), currentData);
  assert.equal(icons.length, iconUpdates);
  assert.equal(enabled.length, enabledUpdates);
  controller.disconnect();
  assert.equal(socket.closeCalls, 1);
});

test("destroying monitoring closes its socket without requiring an abstract base onExit implementation", () => {
  const { controller } = monitoringFixture();
  const socket = controller.socket;
  assert.equal(controller.onExit(), undefined);
  assert.equal(controller.socket, undefined);
  assert.equal(socket.closeCalls, 1);
  for (const handler of ["onopen", "onclose", "onmessage", "onerror"]) assert.equal(socket[handler], null);
});

test("the active monitoring socket retains connection status and metric formatting", () => {
  const { controller, icons, enabled } = monitoringFixture();
  const socket = controller.socket;
  assert.equal(socket.url, "wss://test.invalid/api/monitoring");
  assert.equal(enabled.at(-1), false);
  assert.equal(icons.at(-1), "sap-icon://disconnected");
  socket.readyState = FakeWebSocket.OPEN;
  socket.onopen({});
  assert.equal(enabled.at(-1), true);
  assert.equal(icons.at(-1), "sap-icon://connected");
  socket.onmessage({
    data: JSON.stringify({
      Update: {
        monitoring: {
          db: { caches: { accesses: 10, hits: 7, misses: 3, entries: 1, hitRate: 70 } },
          app: { memCurrent: 2048, memMax: 4096 },
          sys: { uptime: { secs: 123 } }
        }
      }
    })
  });
  const rows = controller.models.get("monitoring").getData();
  assert.equal(rows.find((row) => row.name === "monitoring.caches.hitRate").value, "70.0 %");
  assert.equal(rows.find((row) => row.name === "monitoring.app.mem_current").value, "2.0 KiB");
  assert.equal(rows.find((row) => row.name === "monitoring.sys.uptime").value, "2m 3s");
  const message = socket.onmessage;
  socket.readyState = FakeWebSocket.CLOSED;
  socket.onclose({});
  assert.equal(controller.socket, undefined);
  assert.equal(enabled.at(-1), true);
  assert.equal(icons.at(-1), "sap-icon://disconnected");
  message({ data: JSON.stringify({ Update: { monitoring: {} } }) });
  assert.equal(controller.models.get("monitoring").getData(), rows);
});

test("real base-table cleanup tolerates UI5's absent abstract onExit hook", () => {
  const controller = new (loadController("BaseTable"))();
  assert.equal(controller.onExit(), undefined);

  const original = BaseController.prototype.onExit;
  const cleaned = [];
  BaseController.prototype.onExit = function () {
    cleaned.push(this);
  };
  try {
    assert.equal(controller.onExit(), undefined);
    const { controller: monitoring } = monitoringFixture();
    assert.equal(monitoring.onExit(), undefined);
    assert.deepEqual(cleaned, [controller, monitoring], "implemented base cleanup must still run");
  } finally {
    if (original === undefined) delete BaseController.prototype.onExit;
    else BaseController.prototype.onExit = original;
  }
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
  assert.equal(controller.routes.get(Routes.HeatDetails)(event), undefined);
  event.getParameter = () => {
    throw new Error("Event was already cleared");
  };
  assert.deepEqual(loads, [47]);
  await setImmediate();
  assert.deepEqual(controller.feedback, []);
});

test("club athlete filters capture and apply the club ID before loading", async () => {
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
  assert.equal(controller.routes.get(Routes.ClubAthletes)(event), undefined);
  event.getParameter = () => {
    throw new Error("Event was already cleared");
  };
  assert.deepEqual(filters, [7]);
  resolve(true);
  await setImmediate();
  assert.deepEqual(filters, [7]);
});

function athletesFixture(athletes = []) {
  const controller = new (loadController("AthletesTable"))();
  let applied = [];
  const table = { getBinding: () => ({ filter: (filters) => (applied = filters) }) };
  const comboBox = {
    selectedKey: "",
    value: "",
    setSelectedKey(key) {
      this.selectedKey = key;
      const item = controller.models
        .get("clubFilter")
        .getData()
        .find((choice) => choice.key === key);
      // Match UI5: unknown keys retain the old visual selection unless cleared first.
      if (!key) this.value = "";
      else if (item) this.value = item.text;
      return this;
    }
  };
  controller.view.byId = (id) => (id === "athletesTable" ? table : comboBox);
  controller.component.getActiveRegatta = () => Promise.resolve(new JSONModel({ id: 7 }));
  controller.onInit();
  const model = controller.models.get("athletes");
  model.loadData = async (url) => {
    assert.equal(url, "/api/regattas/7/athletes");
    model.setData(athletes);
  };
  const match = (clubId) =>
    controller.routes.get(clubId === undefined ? Routes.Athletes : Routes.ClubAthletes)({
      getParameter: () => (clubId === undefined ? {} : { clubId: String(clubId) })
    });
  const visibleIds = () =>
    Array.from(model.getData())
      .filter((athlete) => applied.every((filter) => matchesClubSearch(filter, athlete)))
      .map((athlete) => athlete.id);
  const select = (key) =>
    controller.onClubSelectionChange({
      getParameters: () => ({ selectedItem: key ? { getKey: () => key } : undefined })
    });
  return { controller, comboBox, model, match, visibleIds, select };
}

test("club athlete routes replace A with empty B, including during loading and on failure", async () => {
  const { controller, comboBox, model, match, visibleIds } = athletesFixture([
    { id: 11, club: { id: 1, shortName: "Club A" } }
  ]);
  match(1);
  await setImmediate();
  assert.deepEqual(visibleIds(), [11]);
  assert.equal(comboBox.value, "Club A");
  model.loadData = () => Promise.reject({ statusCode: 503, statusText: "Unavailable" });
  match(2);
  assert.equal(controller.clubFilter.getPath(), "club/id");
  assert.equal(controller.clubFilter.getValue1(), 2);
  assert.deepEqual(visibleIds(), []);
  assert.equal(comboBox.selectedKey, "2");
  assert.equal(comboBox.value, "", "missing clubs must not retain another club's label");
  await setImmediate();
  assert.deepEqual(visibleIds(), []);
  match();
  assert.deepEqual(visibleIds(), [11]);
  assert.equal(comboBox.selectedKey, "");
  await setImmediate();
});

test("a fresh empty-club route stays restricted after a successful athlete load", async () => {
  const { controller, comboBox, match, visibleIds } = athletesFixture([
    { id: 11, club: { id: 1, shortName: "Club A" } }
  ]);
  match(2);
  assert.equal(controller.clubFilter.getValue1(), 2);
  await setImmediate();
  assert.deepEqual(visibleIds(), []);
  assert.equal(comboBox.selectedKey, "2");
  assert.equal(comboBox.value, "");
});

test("equal club names retain separate ID choices and restrict routes and manual selections by ID", async () => {
  const { controller, comboBox, match, visibleIds, select } = athletesFixture([
    { id: 11, lastName: "Alice", club: { id: 1, shortName: "Same club" } },
    { id: 12, lastName: "Bob", club: { id: 1, shortName: "Same club" } },
    { id: 21, lastName: "Alice", club: { id: 2, shortName: "Same club" } }
  ]);
  match(1);
  await setImmediate();
  const choices = Array.from(controller.models.get("clubFilter").getData());
  assert.deepEqual(
    choices.map((choice) => choice.key),
    ["1", "2"]
  );
  assert.notEqual(choices[0].text, choices[1].text);
  assert.equal(comboBox.selectedKey, "1");
  assert.deepEqual(visibleIds(), [11, 12]);
  match(2);
  assert.deepEqual(visibleIds(), [21]);
  await setImmediate();
  select("1");
  assert.deepEqual(visibleIds(), [11, 12]);
  controller.onSearchFieldLiveChange({ getParameters: () => ({ newValue: " Alice " }) });
  assert.deepEqual(visibleIds(), [11]);
  select("2");
  assert.deepEqual(visibleIds(), [21]);
  select("");
  assert.deepEqual(visibleIds(), [11, 21], "clearing the club must preserve text search");
  assert.equal(comboBox.selectedKey, "");
});

test("late athlete loads use current club selection and cannot restore an obsolete route restriction", async () => {
  for (const [firstClub, currentClub] of [
    [1, 2],
    [undefined, 2],
    [1, undefined]
  ]) {
    const athletes = [
      { id: 11, club: { id: 1, shortName: "Club A" } },
      { id: 21, club: { id: 2, shortName: "Club B" } }
    ];
    const { comboBox, model, match, visibleIds } = athletesFixture(athletes);
    const pending = [];
    model.loadData = () => new Promise((resolve) => pending.push(resolve));
    match(firstClub);
    await setImmediate();
    match(currentClub);
    await setImmediate();
    model.setData(athletes);
    pending[1]();
    await setImmediate();
    pending[0]();
    await setImmediate();
    assert.equal(comboBox.selectedKey, currentClub === undefined ? "" : String(currentClub));
    assert.equal(comboBox.value, currentClub === undefined ? "" : "Club B");
    assert.deepEqual(visibleIds(), currentClub === undefined ? [11, 21] : [21]);
  }
});

test("refresh rebuilds choices without losing the current club restriction", async () => {
  const { controller, comboBox, model, match, visibleIds } = athletesFixture([
    { id: 11, club: { id: 1, shortName: "Club A" } }
  ]);
  match(1);
  await setImmediate();
  const enabled = [];
  model.loadData = async () => model.setData([{ id: 21, club: { id: 2, shortName: "Club B" } }]);
  await controller.onRefreshButtonPress({ getSource: () => ({ setEnabled: (value) => enabled.push(value) }) });
  assert.deepEqual(enabled, [false, true]);
  assert.deepEqual(controller.feedback, [true]);
  assert.equal(comboBox.selectedKey, "1");
  assert.equal(comboBox.value, "");
  assert.deepEqual(visibleIds(), []);
});

test("club-entry crew search safely rejects omitted, empty, and malformed crews", () => {
  const controller = new (loadController("ClubDetails"))();
  const crewMatches = controller.createSearchFilters("alice")[0].getFilters()[0].getTest();
  for (const crews of [
    undefined,
    null,
    [],
    {},
    "not an array",
    [null, undefined, {}],
    [{ athlete: null }],
    [{ athlete: { firstName: 42, lastName: null } }]
  ]) {
    assert.equal(crewMatches(crews), false);
  }
});

test("club-entry crew search matches first or last names case-insensitively", () => {
  const controller = new (loadController("ClubDetails"))();
  const crewMatches = controller.createSearchFilters("ALIce")[0].getFilters()[0].getTest();
  assert.equal(crewMatches([{ athlete: { firstName: "Alice" } }]), true);
  assert.equal(crewMatches([{ athlete: { lastName: "MacAlice" } }]), true);
  assert.equal(crewMatches([null, {}, { athlete: { firstName: null, lastName: "ALICE" } }]), true);
  assert.equal(crewMatches([{ athlete: { firstName: "Bob", lastName: "Smith" } }]), false);
});

// Only the predicates, conjunctions, Contains and EQ operators used by these
// club/athlete filters are simulated here; this is not a full UI5 filtering engine.
function matchesClubSearch(filter, entry) {
  const nested = filter.getFilters();
  if (nested) {
    return filter.isAnd()
      ? nested.every((child) => matchesClubSearch(child, entry))
      : nested.some((child) => matchesClubSearch(child, entry));
  }
  const value = filter
    .getPath()
    .split("/")
    .filter(Boolean)
    .reduce((object, key) => object?.[key], entry);
  const predicate = filter.getTest();
  if (predicate) return predicate(value);
  if (filter.getOperator() === "EQ") return value === filter.getValue1();
  assert.equal(filter.getOperator(), "Contains");
  return value != null && String(value).toLowerCase().includes(String(filter.getValue1()).toLowerCase());
}

test("club-entry search retains race-number matches without crew and clears on blank text", () => {
  const controller = new (loadController("ClubDetails"))();
  const entries = [
    { id: 1, race: { number: "202" } },
    { id: 2, race: { number: "303" }, crew: [] },
    { id: 3, race: { number: "404" }, crew: [{ athlete: { firstName: "Rower 202" } }] },
    { id: 4, race: { number: "505" }, crew: null },
    { id: 5, race: { number: "606" }, crew: [null, {}] }
  ];
  let applied;
  let matched;
  controller.table = {
    getBinding: () => ({
      filter: (filters) => {
        applied = filters;
        matched = entries.filter((entry) => filters.every((filter) => matchesClubSearch(filter, entry)));
      }
    })
  };
  controller.onSearchFieldLiveChange({ getParameters: () => ({ newValue: " 202 " }) });
  assert.equal(applied[0].isAnd(), false);
  const raceFilter = applied[0].getFilters()[1];
  assert.equal(raceFilter.getPath(), "race/number");
  assert.equal(raceFilter.getValue1(), "202");
  assert.deepEqual(
    matched.map((entry) => entry.id),
    [1, 3]
  );
  controller.onSearchFieldLiveChange({ getParameters: () => ({ newValue: "   " }) });
  assert.equal(applied.length, 0);
  assert.deepEqual(matched, entries);
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
  const route = routing.routes.find((item) => item.name === Routes.Notifications);
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

test("route constants exactly match every manifest route name", () => {
  const manifest = JSON.parse(readFileSync(new URL("../webapp/manifest.json", import.meta.url), "utf8"));
  const names = Object.values(Routes);
  assert.equal(names.length, new Set(names).size, "route names must be unique");
  assert.deepEqual(names.sort(), manifest["sap.ui5"].routing.routes.map((route) => route.name).sort());
});

test("controller navigation and route lookups do not use string literals", () => {
  const directory = new URL("../webapp/controller/", import.meta.url);
  let checked = 0;
  for (const file of readdirSync(directory).filter((name) => name.endsWith(".ts"))) {
    const source = ts.createSourceFile(
      file,
      readFileSync(new URL(file, directory), "utf8"),
      ts.ScriptTarget.Latest,
      true
    );
    const isLiteral = (node) => node && (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node));
    const visit = (node) => {
      if (
        ts.isCallExpression(node) &&
        ts.isPropertyAccessExpression(node.expression) &&
        ["navTo", "getRoute", "navBack"].includes(node.expression.name.text)
      ) {
        assert.ok(!isLiteral(node.arguments[0]), `${file}: ${node.getText()}`);
        if (ts.isPropertyAccessExpression(node.arguments[0]) && node.arguments[0].expression.getText() === "Routes") {
          assert.ok(Object.hasOwn(Routes, node.arguments[0].name.text), `${file}: unknown route constant`);
        }
        checked++;
      }
      if (ts.isPropertyAssignment(node) && node.name.getText() === "back") {
        assert.ok(!isLiteral(node.initializer), `${file}: navigation back target must use a route constant`);
      }
      ts.forEachChild(node, visit);
    };
    visit(source);
  }
  assert.ok(checked >= 40, "expected all controller routing calls to be checked");
});

test("notification translation keys are migrated consistently in every locale", () => {
  const directory = new URL("../webapp/i18n/", import.meta.url);
  const locales = readdirSync(directory).filter((file) => file.endsWith(".properties"));
  assert.ok(locales.length >= 2);
  const migrated = (key) =>
    key.startsWith("notifications.") || key === "launchpad.administration" || key === "common.actions";
  const bundles = locales.map((file) => {
    const entries = readFileSync(new URL(file, directory), "utf8")
      .split(/\r?\n/)
      .flatMap((line) => {
        const match = line.match(/^([^#!\s][^=]*)=(.*)$/);
        return match ? [[match[1].trim(), match[2]]] : [];
      });
    const bundle = new Map(entries);
    assert.ok(![...bundle.keys()].some((key) => key.startsWith("admin.")), file);
    const migratedEntries = entries.filter(([key]) => migrated(key));
    assert.equal(
      migratedEntries.length,
      new Set(migratedEntries.map(([key]) => key)).size,
      `duplicate keys in ${file}`
    );
    for (const key of ["notifications.title", "launchpad.administration", "common.actions"]) {
      assert.ok(bundle.get(key), `${file}: ${key}`);
    }
    return { file, bundle };
  });
  const expected = [...bundles[0].bundle.keys()].filter(migrated).sort();
  for (const { file, bundle } of bundles) assert.deepEqual([...bundle.keys()].filter(migrated).sort(), expected, file);

  let checked = 0;
  for (const file of [
    "controller/Notifications.controller.ts",
    "view/Notifications.view.xml",
    "view/Launchpad.view.xml",
    "view/TimekeepingTimestrip.fragment.xml"
  ]) {
    const source = readFileSync(new URL(`../webapp/${file}`, import.meta.url), "utf8");
    const references = [
      ...Array.from(source.matchAll(/i18n\(["']([^"']+)["']/g), (match) => match[1]),
      ...Array.from(source.matchAll(/\b(?:success|error):\s*["'](notifications\.[^"']+)["']/g), (match) => match[1]),
      ...Array.from(source.matchAll(/i18n>([^}]+)\}/g), (match) => match[1])
    ];
    assert.ok(!references.some((key) => key.startsWith("admin.")), file);
    for (const key of references.filter(migrated)) {
      for (const { file: locale, bundle } of bundles)
        assert.ok(bundle.get(key), `${file}: ${key} missing in ${locale}`);
      checked++;
    }
  }
  assert.ok(checked > 25, "expected controller and XML translation coverage");
});
