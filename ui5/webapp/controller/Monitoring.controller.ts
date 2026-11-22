import Button from "sap/m/Button";
import GroupHeaderListItem from "sap/m/GroupHeaderListItem";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import JSONModel from "sap/ui/model/json/JSONModel";
import BaseController from "./Base.controller";
import { Routes } from "../model/Routes";

/**
 * Live monitoring dashboard mapping WebSocket snapshots to localized, grouped metric rows.
 *
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class MonitoringController extends BaseController {
  private readonly units = ["bytes", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB"];
  private readonly monitoringModel: JSONModel = new JSONModel();
  private socket?: WebSocket;
  private statusButton?: Button;

  /** Registers the monitoring model, route-triggered connection, and disconnect-on-hide delegate. */
  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.getView()?.addEventDelegate({ onBeforeHide: this.onBeforeHide }, this);
    super
      .getRouter()
      ?.getRoute(Routes.Monitoring)
      ?.attachMatched((_: Route$MatchedEvent) => this.connect(), this);
    super.setViewModel(this.monitoringModel, "monitoring");
    this.statusButton = this.byId("statusButton") as Button | undefined;
  }

  private onBeforeHide(): void {
    this.disconnect();
  }

  /** Closes the monitoring stream on view destruction and runs any implemented base cleanup. */
  onExit(): void {
    this.disconnect();
    super.onExit?.();
  }

  /** Returns to the launchpad, closes the stream, and clears displayed metrics. */
  onNavBack(): void {
    super.navToStartPage();
    this.disconnect();
    this.updateModel({});
  }

  onStatusButtonPress(): void {
    this.connect();
  }

  /**
   * Flattens available database, application, and system metrics into localized rows grouped by category.
   *
   * @param monitoring - Monitoring snapshot with optional `db`, `app`, and `sys` categories.
   * An empty object clears the displayed rows.
   */
  private updateModel(monitoring: any) {
    const all = [];

    if (monitoring?.db?.connections) {
      all.push(
        { name: this.i18n("monitoring.dbConnections.total"), value: monitoring.db.connections.total, group: "1" },
        { name: this.i18n("monitoring.dbConnections.used"), value: monitoring.db.connections.used, group: "1" },
        { name: this.i18n("monitoring.dbConnections.idle"), value: monitoring.db.connections.idle, group: "1" },
        { name: this.i18n("monitoring.dbConnections.created"), value: monitoring.db.connections.created, group: "1" },
        {
          name: this.i18n("monitoring.dbConnections.closedIdleTimeout"),
          value: monitoring.db.connections.closedIdleTimeout,
          group: "1"
        },
        {
          name: this.i18n("monitoring.dbConnections.closedMaxLifetime"),
          value: monitoring.db.connections.closedMaxLifetime,
          group: "1"
        },
        {
          name: this.i18n("monitoring.dbConnections.closedError"),
          value: monitoring.db.connections.closedError,
          group: "1"
        }
      );
    }

    if (monitoring?.db?.userPools) {
      monitoring.db.userPools.forEach((pool: { username: string; sessions: number }) => {
        all.push({ name: pool.username, value: pool.sessions, group: "2" });
      });
    }

    if (monitoring?.db?.caches) {
      all.push(
        { name: this.i18n("monitoring.caches.accesses"), value: monitoring.db.caches.accesses, group: "3" },
        { name: this.i18n("monitoring.caches.hits"), value: monitoring.db.caches.hits, group: "3" },
        { name: this.i18n("monitoring.caches.misses"), value: monitoring.db.caches.misses, group: "3" },
        { name: this.i18n("monitoring.caches.entries"), value: monitoring.db.caches.entries, group: "3" },
        {
          name: this.i18n("monitoring.caches.hitRate"),
          value: this.nicePercent(monitoring.db.caches.hitRate),
          group: "3"
        }
      );
    }

    if (monitoring?.app) {
      all.push(
        { name: this.i18n("monitoring.app.mem_current"), value: this.niceBytes(monitoring.app.memCurrent), group: "4" },
        { name: this.i18n("monitoring.app.mem_max"), value: this.niceBytes(monitoring.app.memMax), group: "4" }
      );
    }

    if (monitoring?.sys?.mem) {
      all.push(
        { name: this.i18n("monitoring.mem.total"), value: this.niceBytes(monitoring.sys.mem.total), group: "5" },
        { name: this.i18n("monitoring.mem.used"), value: this.niceBytes(monitoring.sys.mem.used), group: "5" },
        {
          name: this.i18n("monitoring.mem.available"),
          value: this.niceBytes(monitoring.sys.mem.available),
          group: "5"
        },
        { name: this.i18n("monitoring.mem.free"), value: this.niceBytes(monitoring.sys.mem.free), group: "5" }
      );
    }

    if (monitoring?.sys?.cpus) {
      monitoring.sys.cpus.forEach((cpu: any, _index: number) => {
        all.push({ name: cpu.name, value: this.nicePercent(cpu.usage), group: "6" });
      });
    }

    if (monitoring?.sys?.uptime) {
      all.push({
        name: this.i18n("monitoring.sys.uptime"),
        value: this.niceDuration(monitoring.sys.uptime.secs),
        group: "7"
      });
    }

    this.monitoringModel.setData(all);
  }

  /**
   * UI5 grouping callback returning the metric-category key from a row's binding context.
   *
   * @param context - Binding context of the metric row being grouped.
   * @returns The row's `group` category key used by the group-header factory.
   */
  getGroup(context: any): string {
    return context.getProperty("group");
  }

  /**
   * UI5 group-header factory localizing metric-category keys 1 through 7.
   *
   * @param group - UI5 grouping descriptor whose `key` identifies the metric category.
   * @returns A group-header list item with the localized category title, or an empty title for an unknown key.
   */
  getGroupHeader(group: any): any {
    let title: string = "";
    switch (group?.key) {
      case "1":
        title = this.i18n("monitoring.dbConnections.title");
        break;
      case "2":
        title = this.i18n("monitoring.userPools.title");
        break;
      case "3":
        title = this.i18n("monitoring.caches.title");
        break;
      case "4":
        title = this.i18n("monitoring.app.title");
        break;
      case "5":
        title = this.i18n("monitoring.mem.title");
        break;
      case "6":
        title = this.i18n("monitoring.cpus.title");
        break;
      case "7":
        title = this.i18n("monitoring.sys.title");
        break;
    }
    return new GroupHeaderListItem({
      title: title
    });
  }

  /**
   * Formats an already percentage-scaled value with one decimal place; does not multiply by 100.
   *
   * @param n - Percentage value, such as 42.5 for 42.5 percent, not a fractional ratio.
   * @returns The value rounded to one decimal place followed by ` %`.
   */
  private nicePercent(n: number): string {
    return n.toFixed(1) + " %";
  }

  /**
   * Scales bytes by powers of 1024, using one decimal place only for scaled values below 10.
   *
   * @param n - Byte count to scale using binary units.
   * @returns Rounded byte text followed by a unit from `bytes` through `YiB` for values in the supported range.
   */
  private niceBytes(n: number): string {
    let l: number = 0;
    while (n >= 1024 && ++l) {
      n = n / 1024;
    }
    return n.toFixed(n < 10 && l > 0 ? 1 : 0) + " " + this.units[l];
  }

  /**
   * Formats elapsed seconds as nonzero day/hour/minute components followed by whole seconds.
   *
   * @param seconds - Elapsed duration in seconds; fractional seconds are rounded down.
   * @returns A compact duration such as `1d 2h 3m 4s`, always including the seconds component.
   */
  private niceDuration(seconds: number): string {
    const days: number = Math.floor(seconds / 60 / 60 / 24);
    const hours: number = Math.floor((seconds / 60 / 60) % 24);
    const minutes: number = Math.floor((seconds / 60) % 60);
    const secs: number = Math.floor(seconds % 60);
    const duration: string =
      (days > 0 ? days + "d " : "") +
      (hours > 0 ? hours + "h " : "") +
      (minutes > 0 ? minutes + "m " : "") +
      secs +
      "s";
    return duration;
  }

  /**
   * Replaces the monitoring stream, using WSS on HTTPS pages and disabling reconnect until open or close.
   * Only callbacks from the currently owned socket may update metrics or connection status.
   */
  private connect() {
    this.statusButton?.setEnabled(false);

    this.disconnect();
    this.statusButton?.setIcon("sap-icon://disconnected");

    const location: Location = globalThis.location;
    const proto = location.protocol.startsWith("https") ? "wss" : "ws";

    console.debug("Connecting Monitoring WebSocket ...");
    const socket = new WebSocket(`${proto}://${location.host}/api/monitoring`);
    this.socket = socket;

    socket.onopen = (_event: Event) => {
      if (this.socket !== socket) {
        return;
      }
      this.statusButton?.setEnabled(true);
      this.statusButton?.setIcon("sap-icon://connected");
      console.debug("Monitoring WebSocket Connected");
    };
    socket.onclose = (_event: CloseEvent) => {
      if (this.socket !== socket) {
        return;
      }
      delete this.socket;
      this.statusButton?.setEnabled(true);
      this.statusButton?.setIcon("sap-icon://disconnected");
      console.debug("Monitoring WebSocket Disconnected");
    };
    socket.onmessage = (event: MessageEvent) => {
      if (this.socket !== socket) {
        return;
      }
      const monitoring: any = JSON.parse(event.data);
      if (monitoring.Update) {
        this.updateModel(monitoring.Update.monitoring);
      } else console.warn("Unknown Monitoring WebSocket Message", monitoring);
    };
  }

  /** Invalidates socket ownership and detaches callbacks before closing; safe to call more than once. */
  private disconnect() {
    const socket = this.socket;
    if (socket) {
      delete this.socket;
      socket.onopen = null;
      socket.onclose = null;
      socket.onmessage = null;
      socket.onerror = null;
      console.debug("Disconnecting Monitoring WebSocket ...");
      socket.close();
    }
  }
}
