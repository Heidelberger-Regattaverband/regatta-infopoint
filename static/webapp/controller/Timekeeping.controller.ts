import Button, { Button$PressEvent } from "sap/m/Button";
import MessageBox from "sap/m/MessageBox";
import { ListBase$SelectionChangeEvent } from "sap/m/ListBase";
import ListItemBase from "sap/m/ListItemBase";
import Table from "sap/m/Table";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import Context from "sap/ui/model/Context";
import JSONModel from "sap/ui/model/json/JSONModel";
import Formatter from "../model/Formatter";
import BaseTableController from "./BaseTable.controller";
import IconPool from "sap/ui/core/IconPool";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class TimekeepingController extends BaseTableController {

  private static readonly TIMESTRIP_MODEL: string = "timestrip";
  private static readonly AQUARIUS_HEATS_MODEL: string = "aquariusHeats";

  readonly formatter: Formatter = Formatter;
  private socket?: WebSocket;
  private statusButton?: Button;
  // bind keyListener method to this context to have access to navigation methods
  private readonly keyListener: (event: KeyboardEvent) => void = this.onKeyDown.bind(this);

  onInit(): void {
    super.init(super.getView()?.byId("timestripTable") as Table, "timestamp" /* eventBus channel */);
    this.statusButton = this.byId("timekeepingStatusButton") as Button;
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.getView()?.addEventDelegate({ onBeforeShow: this.onBeforeShow, onBeforeHide: this.onBeforeHide }, this);
    super.setViewModel(new JSONModel(), TimekeepingController.TIMESTRIP_MODEL);
    super.setViewModel(new JSONModel(), TimekeepingController.AQUARIUS_HEATS_MODEL);
    super.getRouter()?.getRoute("timekeeping")?.attachMatched((_: Route$MatchedEvent) => {
      this.connect();
      this.loadTimestripModel();
    }, this);
    this._loadIcons();
  }

  private _loadIcons(): void {
    const fonts: any[] = [
      {
        fontFamily: "SAP-icons-TNT",
        fontURI: sap.ui.require.toUrl("sap/tnt/themes/base/fonts/")
      }
    ];
    fonts.forEach((font: any) => {
      IconPool.registerFont(font);
    });
  }

  private onBeforeShow(): void {
    globalThis.addEventListener("keydown", this.keyListener);
  }

  private onBeforeHide(): void {
    globalThis.removeEventListener("keydown", this.keyListener);
    this.disconnect();
  }

  onSelectionChange(event: ListBase$SelectionChangeEvent): void {
    const selectedItem: ListItemBase | undefined = event.getParameter("listItem");
    if (selectedItem) {
      const bindingCtx: Context | null | undefined = selectedItem.getBindingContext(TimekeepingController.TIMESTRIP_MODEL);
      const timestamp: any = bindingCtx?.getModel().getProperty(bindingCtx.getPath());
      this.onItemChanged(timestamp);
    }
  }

  onNavBack(): void {
    super.navToStartPage();
    // reduce table growing threshold to improve performance next time table is shown
    this.table.setGrowingThreshold(30);
  }

  onStatusButtonPress(): void {
    this.connect();
  }

  onStartButtonPress(): void {
    this.sendCommand({ AddStart: null });
  }

  onFinishButtonPress(): void {
    this.sendCommand({ AddFinish: null });
  }

  onRefreshButtonPress(event: Button$PressEvent): void {
    this.loadTimestripModel();
  }

  onDeleteTimestamp(event: Button$PressEvent): void {
    const bindingCtx: Context | null | undefined = event.getSource().getBindingContext(TimekeepingController.TIMESTRIP_MODEL);
    if (bindingCtx) {
      const timestamp: any = bindingCtx.getModel().getProperty(bindingCtx.getPath());
      if (timestamp?.time) {
        MessageBox.confirm(super.i18n("timekeeping.deleteTimestamp.message"), {
          title: super.i18n("timekeeping.deleteTimestamp.title"),
          emphasizedAction: MessageBox.Action.CANCEL,
          onClose: (action: any) => {
            if (action === MessageBox.Action.OK) {
              this.sendCommand({ DeleteTimestamp: { time: timestamp.time } });
              this.deleteTimestamp(timestamp);
            }
          }
        });
      }
    }
  }

  onItemChanged(item: any): void {
  }

  private loadTimestripModel() {
    this.sendCommand({ GetTimestrip: null });
  }

  private updateModel(timekeeping: any) {
    // nothing to do yet
  }

  private connect() {
    this.disconnect();

    const location: Location = globalThis.location;
    const proto = location.protocol.startsWith('https') ? 'wss' : 'ws';

    console.debug('Connecting Timekeeping WebSocket ...');
    this.socket = new WebSocket(`${proto}://${location.host}/api/timekeeping`);

    this.socket.onopen = (_event: Event) => {
      this.statusButton?.setIcon('sap-icon://connected');
      console.debug('Timekeeping WebSocket Connected');
    }
    this.socket.onclose = (_event: CloseEvent) => {
      this.statusButton?.setIcon('sap-icon://disconnected');
      console.debug('Timekeeping WebSocket Disconnected');
    }
    this.socket.onmessage = (msgEvent: MessageEvent) => {
      const data: any = JSON.parse(msgEvent.data);
      if (data.error) {
        console.error('Timekeeping WebSocket error:', data.error);
        super.showErrorMessageToast(data.error);
      } else
        if (data.AquariusHeats) {
          super.getViewJSONModel(TimekeepingController.AQUARIUS_HEATS_MODEL).setData(data.AquariusHeats.heats);
        } else if (data.Timestamp) {
          this.updateTimestamp(data.Timestamp.timestamp);
          super.showInfoMessageToast("Timestamp added successfully");
        } else if (data.Timestrip) {
          super.getViewJSONModel(TimekeepingController.TIMESTRIP_MODEL).setData(data.Timestrip.time_stamps);
          super.showInfoMessageToast("Timestrip retrieved successfully");
        } else {
          console.warn(`Received unknown Timekeeping WebSocket event type: ${JSON.stringify(data)}`);
        }
    }
  }

  private updateTimestamp(timestamp: any) {
    const timestripModel: JSONModel = super.getViewJSONModel(TimekeepingController.TIMESTRIP_MODEL);
    const existingTimestamps: any[] = timestripModel.getData() || [];
    const timestampIndex: number = existingTimestamps.findIndex((t: any) => t.time === timestamp.time);
    if (timestampIndex >= 0) {
      existingTimestamps[timestampIndex] = timestamp;
    } else {
      existingTimestamps.push(timestamp);
    }
    timestripModel.setData(existingTimestamps);
  }

  private deleteTimestamp(timestamp: any) {
    const timestripModel: JSONModel = super.getViewJSONModel(TimekeepingController.TIMESTRIP_MODEL);
    const existingTimestamps: any[] = timestripModel.getData() || [];
    const timestampIndex: number = existingTimestamps.findIndex((t: any) => t.time === timestamp.time);
    if (timestampIndex >= 0) {
      existingTimestamps.splice(timestampIndex, 1);
    }
    timestripModel.setData(existingTimestamps);
  }

  private disconnect() {
    if (this.socket) {
      console.debug('Disconnecting Timekeeping WebSocket ...');
      this.socket.close();
      delete this.socket;
    }
  }

  private sendCommand(command: any) {
    if (this.socket?.readyState === WebSocket.OPEN) {
      this.socket.send(JSON.stringify(command));
    }
  }

  private onKeyDown(event: KeyboardEvent): void {
    switch (event.key) {
      case " ":
        this.sendCommand({ AddFinish: null });
        event.stopPropagation();
        event.preventDefault();
        event.stopImmediatePropagation();
        break;
      case "+":
        this.sendCommand({ AddStart: null });
        event.stopPropagation();
        event.preventDefault();
        event.stopImmediatePropagation();
        break;
      default:
        break;
    }
  }
}
