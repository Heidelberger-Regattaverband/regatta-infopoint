import Button from "sap/m/Button";
import GroupHeaderListItem from "sap/m/GroupHeaderListItem";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import JSONModel from "sap/ui/model/json/JSONModel";
import BaseController from "./Base.controller";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class MonitoringController extends BaseController {

  private readonly units = ['bytes', 'KiB', 'MiB', 'GiB', 'TiB', 'PiB', 'EiB', 'ZiB', 'YiB'];
  private readonly monitoringModel: JSONModel = new JSONModel();
  private socket?: WebSocket;
  private statusButton?: Button;

  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.getView()?.addEventDelegate({ onBeforeHide: this.onBeforeHide }, this);
    super.getRouter()?.getRoute("monitoring")?.attachMatched((_: Route$MatchedEvent) => this.connect(), this);
    super.setViewModel(this.monitoringModel, "monitoring");
    this.statusButton = this.byId("statusButton") as Button | undefined;
  }

  private onBeforeHide(): void {
    this.disconnect();
  }

  onNavBack(): void {
    super.navToStartPage();
    this.disconnect();
    this.updateModel({});
  }

  onStatusButtonPress(): void {
    this.connect();
  }

  private updateModel(monitoring: any) {
    const all = [];

    if (monitoring?.db?.connections) {
      all.push({ name: this.i18n("monitoring.dbConnections.total"), value: monitoring.db.connections.total, group: "1" },
        { name: this.i18n("monitoring.dbConnections.used"), value: monitoring.db.connections.used, group: "1" },
        { name: this.i18n("monitoring.dbConnections.idle"), value: monitoring.db.connections.idle, group: "1" },
        { name: this.i18n("monitoring.dbConnections.created"), value: monitoring.db.connections.created, group: "1" },
        { name: this.i18n("monitoring.dbConnections.closedIdleTimeout"), value: monitoring.db.connections.closedIdleTimeout, group: "1" },
        { name: this.i18n("monitoring.dbConnections.closedMaxLifetime"), value: monitoring.db.connections.closedMaxLifetime, group: "1" },
        { name: this.i18n("monitoring.dbConnections.closedError"), value: monitoring.db.connections.closedError, group: "1" });
    }

    if (monitoring?.db?.caches) {
      all.push({ name: this.i18n("monitoring.caches.hits"), value: monitoring.db.caches.hits, group: "2" },
        { name: this.i18n("monitoring.caches.misses"), value: monitoring.db.caches.misses, group: "2" },
        { name: this.i18n("monitoring.caches.entries"), value: monitoring.db.caches.entries, group: "2" },
        { name: this.i18n("monitoring.caches.hitRate"), value: this.nicePercent(monitoring.db.caches.hitRate), group: "2" });
    }

    if (monitoring?.app) {
      all.push({ name: this.i18n("monitoring.app.mem_current"), value: this.niceBytes(monitoring.app.mem_current), group: "3" },
        { name: this.i18n("monitoring.app.mem_max"), value: this.niceBytes(monitoring.app.mem_max), group: "3" });
    }

    if (monitoring?.sys?.mem) {
      all.push({ name: this.i18n("monitoring.mem.total"), value: this.niceBytes(monitoring.sys.mem.total), group: "4" },
        { name: this.i18n("monitoring.mem.used"), value: this.niceBytes(monitoring.sys.mem.used), group: "4" },
        { name: this.i18n("monitoring.mem.available"), value: this.niceBytes(monitoring.sys.mem.available), group: "4" },
        { name: this.i18n("monitoring.mem.free"), value: this.niceBytes(monitoring.sys.mem.free), group: "4" });
    }

    if (monitoring?.sys?.cpus) {
      monitoring.sys.cpus.forEach((cpu: any, _index: number) => {
        all.push({ name: cpu.name, value: this.nicePercent(cpu.usage), group: "5" });
      });
    }

    if (monitoring?.sys?.uptime) {
      all.push({ name: this.i18n("monitoring.sys.uptime"), value: this.niceDuration(monitoring.sys.uptime.secs), group: "6" });
    }

    this.monitoringModel.setData(all);
  }

  getGroup(context: any): string {
    return context.getProperty("group");
  }

  getGroupHeader(group: any): any {
    let title: string = "";
    switch (group?.key) {
      case "1":
        title = this.i18n("monitoring.dbConnections.title");
        break;
      case "2":
        title = this.i18n("monitoring.caches.title");
        break;
      case "3":
        title = this.i18n("monitoring.app.title");
        break;
      case "4":
        title = this.i18n("monitoring.mem.title");
        break;
      case "5":
        title = this.i18n("monitoring.cpus.title");
        break;
      case "6":
        title = this.i18n("monitoring.sys.title");
        break;
    }
    return new GroupHeaderListItem({
      title: title,
    });
  }

  private nicePercent(n: number): string {
    return n.toFixed(1) + ' %';
  }

  private niceBytes(n: number): string {
    let l: number = 0;
    while (n >= 1024 && ++l) {
      n = n / 1024;
    }
    return (n.toFixed(n < 10 && l > 0 ? 1 : 0) + ' ' + this.units[l]);
  }

  private niceDuration(seconds: number): string {
    const days: number = Math.floor(seconds / 60 / 60 / 24);
    const hours: number = Math.floor(seconds / 60 / 60 % 24);
    const minutes: number = Math.floor(seconds / 60 % 60);
    const secs: number = Math.floor(seconds % 60);
    const duration: string = (days > 0 ? days + 'd ' : '') + (hours > 0 ? hours + 'h ' : '') + (minutes > 0 ? minutes + 'm ' : '') + secs + 's';
    return duration;
  }

  private connect() {
    this.disconnect();

    const location: Location = globalThis.location;
    const proto = location.protocol.startsWith('https') ? 'wss' : 'ws';

    console.debug('Connecting...');
    this.socket = new WebSocket(`${proto}://${location.host}/api/monitoring`);

    this.socket.onopen = (_event: Event) => {
      console.debug('Connected');
      this.statusButton?.setIcon('sap-icon://connected');
    }

    this.socket.onmessage = (event: MessageEvent) => {
      const monitoring = JSON.parse(event.data);
      this.updateModel(monitoring);
    }

    this.socket.onclose = (_event: CloseEvent) => {
      this.statusButton?.setIcon('sap-icon://disconnected');
      console.debug('Disconnected');
    }
  }

  private disconnect() {
    if (this.socket) {
      this.socket.close();
      delete this.socket;
      console.debug('Disconnecting...');
    }
  }
}