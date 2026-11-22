import Button, { Button$PressEvent } from "sap/m/Button";
import { ComboBox$ChangeEvent } from "sap/m/ComboBox";
import { ComboBoxBase$LoadItemsEvent } from "sap/m/ComboBoxBase";
import { ListBase$SelectionChangeEvent } from "sap/m/ListBase";
import ListItemBase from "sap/m/ListItemBase";
import MessageBox from "sap/m/MessageBox";
import Table from "sap/m/Table";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import Context from "sap/ui/model/Context";
import JSONModel from "sap/ui/model/json/JSONModel";
import ListBinding from "sap/ui/model/ListBinding";
import Formatter from "../model/Formatter";
import { Routes } from "../model/Routes";
import BaseTableController from "./BaseTable.controller";

/**
 * Live timestrip and Aquarius heat view with WebSocket commands for start/finish timestamps and assignments.
 *
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class TimekeepingController extends BaseTableController {
  private static readonly TIMESTRIP_MODEL: string = "timestrip";
  private static readonly AQUARIUS_HEATS_MODEL: string = "aquariusHeats";
  private static readonly HEATS_MODEL: string = "heats";
  private static readonly BIBS_MODEL: string = "bibs";

  readonly formatter: Formatter = Formatter;
  private socket?: WebSocket;
  private statusButton?: Button;
  // bind keyListener method to this context to have access to navigation methods
  private readonly keyListener: (event: KeyboardEvent) => void = this.onKeyDown.bind(this);

  /** Registers timekeeping models, table behavior, visibility-scoped keyboard shortcuts, and route connection. */
  onInit(): void {
    super.init(super.getView()?.byId("timestripTable") as Table, "timestamp" /* eventBus channel */);
    this.statusButton = this.byId("timekeepingStatusButton") as Button;
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.getView()?.addEventDelegate({ onBeforeShow: this.onBeforeShow, onBeforeHide: this.onBeforeHide }, this);
    super.setViewModel(new JSONModel(), TimekeepingController.TIMESTRIP_MODEL);
    super.setViewModel(new JSONModel(), TimekeepingController.AQUARIUS_HEATS_MODEL);
    super.setViewModel(new JSONModel(), TimekeepingController.HEATS_MODEL);
    super.setViewModel(new JSONModel(), TimekeepingController.BIBS_MODEL);
    super
      .getRouter()
      ?.getRoute(Routes.Timekeeping)
      ?.attachMatched((_: Route$MatchedEvent) => {
        this.connect();
      }, this);
  }

  private onBeforeShow(): void {
    globalThis.addEventListener("keydown", this.keyListener);
  }

  /** Removes timekeeping shortcuts and closes the socket when the page is hidden. */
  private onBeforeHide(): void {
    globalThis.removeEventListener("keydown", this.keyListener);
    this.disconnect();
  }

  /**
   * Passes the selected timestrip timestamp to the table's selection hook.
   *
   * @param event - Selection event whose list item is bound to a timestrip timestamp.
   */
  onSelectionChange(event: ListBase$SelectionChangeEvent): void {
    const selectedItem: ListItemBase | undefined = event.getParameter("listItem");
    if (selectedItem) {
      const bindingCtx: Context | null | undefined = selectedItem.getBindingContext(
        TimekeepingController.TIMESTRIP_MODEL
      );
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
    this.sendAddStartCommand();
  }

  onFinishButtonPress(): void {
    this.sendAddFinishCommand();
  }

  /**
   * Requests a fresh timestrip snapshot over the current socket.
   *
   * @param _event - Refresh-button press event; unused because the command needs no source-specific arguments.
   */
  onRefreshButtonPress(_event: Button$PressEvent): void {
    this.sendGetTimestripCommand();
  }

  /**
   * After confirmation, sends a delete command and immediately removes the timestamp from the local timestrip.
   *
   * @param event - Delete-button press event whose binding context identifies the timestamp's time key.
   */
  onDeleteTimestamp(event: Button$PressEvent): void {
    const bindingCtx: Context | null | undefined = event
      .getSource()
      .getBindingContext(TimekeepingController.TIMESTRIP_MODEL);
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

  /**
   * Requests heats ready to start and resumes the combo box's suspended items binding.
   *
   * @param event - Lazy-load event whose source combo box provides the items binding to resume.
   */
  onLoadHeats(event: ComboBoxBase$LoadItemsEvent): void {
    this.sendGetHeatsReadyToStartCommand();
    const binding: ListBinding | undefined = event.getSource().getBinding("items") as ListBinding;
    if (binding) {
      binding.resume();
    } else {
      console.warn("Failed to load heats for timestrip item: No binding found on ComboBox items aggregation");
    }
  }

  /**
   * Placeholder for lazy bib loading; no command is currently sent.
   *
   * @param _event - Bib combo-box lazy-load event; currently unused because bib loading is not implemented.
   */
  onLoadBibs(_event: ComboBoxBase$LoadItemsEvent): void {
    // TODO: Implement loading bibs for timestrip item
  }

  /**
   * Updates the local heat assignment and sends its parsed numeric value to the backend.
   *
   * @param event - Combo-box change event providing the entered heat number and timestamp binding context.
   */
  onHeatChange(event: ComboBox$ChangeEvent): void {
    const heatNr: string = event.getParameter("value") || "";
    const bindingCtx: Context | null | undefined = event
      .getSource()
      .getBindingContext(TimekeepingController.TIMESTRIP_MODEL);
    if (bindingCtx) {
      const timestamp: any = bindingCtx.getModel().getProperty(bindingCtx.getPath());
      if (timestamp) {
        timestamp.heat_nr = heatNr;
        this.sendCommand({ UpdateTimestamp: { time: timestamp.time, heat_nr: Number.parseInt(heatNr, 10) } });
      }
    }
  }

  /**
   * Updates the local bib assignment and sends its parsed numeric value to the backend.
   *
   * @param event - Combo-box change event providing the entered bib number and timestamp binding context.
   */
  onBibChange(event: ComboBox$ChangeEvent): void {
    const bibNr: string = event.getParameter("value") || "";
    const bindingCtx: Context | null | undefined = event
      .getSource()
      .getBindingContext(TimekeepingController.TIMESTRIP_MODEL);
    if (bindingCtx) {
      const timestamp: any = bindingCtx.getModel().getProperty(bindingCtx.getPath());
      if (timestamp) {
        timestamp.bib_nr = bibNr;
        this.sendCommand({ UpdateTimestamp: { time: timestamp.time, bib_nr: Number.parseInt(bibNr, 10) } });
      }
    }
  }

  /**
   * No-op base-table hook; timestamp selection currently has no separate detail view.
   *
   * @param _item - Selected timestamp data; unused until selection-specific behavior is implemented.
   */
  protected onItemChanged(_item: any): void {}

  /**
   * Placeholder for a combined timekeeping snapshot; current socket messages update individual models instead.
   *
   * @param _timekeeping - Timekeeping snapshot; unused because this placeholder has no implementation.
   */
  private updateModel(_timekeeping: any) {
    // nothing to do yet
  }

  /**
   * Replaces the timekeeping socket and requests the full timestrip once connected.
   * Incoming messages update heat/timestamp models or report backend errors.
   */
  private connect() {
    this.disconnect();

    const location: Location = globalThis.location;
    const proto = location.protocol.startsWith("https") ? "wss" : "ws";

    console.debug("Connecting Timekeeping WebSocket ...");
    this.socket = new WebSocket(`${proto}://${location.host}/api/timekeeping`);

    this.socket.onopen = (_event: Event) => {
      this.sendGetTimestripCommand();
      this.statusButton?.setIcon("sap-icon://connected");
      console.debug("Timekeeping WebSocket Connected");
    };
    this.socket.onclose = (_event: CloseEvent) => {
      this.statusButton?.setIcon("sap-icon://disconnected");
      console.debug("Timekeeping WebSocket Disconnected");
    };
    this.socket.onmessage = (msgEvent: MessageEvent) => {
      const data: any = JSON.parse(msgEvent.data);
      if (data.error) {
        console.error("Timekeeping WebSocket error:", data.error);
        super.showErrorMessageToast(data.error);
      } else if (data.AquariusHeats) {
        super.getViewJSONModel(TimekeepingController.AQUARIUS_HEATS_MODEL).setData(data.AquariusHeats.heats);
      } else if (data.Timestamp) {
        this.updateTimestamp(data.Timestamp.timestamp);
        super.showInfoMessageToast("Timestamp added successfully");
      } else if (data.TimeStrip) {
        super.getViewJSONModel(TimekeepingController.TIMESTRIP_MODEL).setData(data.TimeStrip.time_stamps);
        super.showInfoMessageToast("Timestrip retrieved successfully");
      } else if (data.HeatsReadyToStart) {
        super.getViewJSONModel(TimekeepingController.HEATS_MODEL).setData(data.HeatsReadyToStart.heats);
        super.showInfoMessageToast("Heats ready to start retrieved successfully");
      } else {
        console.warn(`Received unknown Timekeeping WebSocket event type: ${JSON.stringify(data)}`);
      }
    };
  }

  /**
   * Replaces a timestamp with the same time key, or prepends a newly received timestamp to the timestrip.
   *
   * @param timestamp - Incoming backend timestamp whose `time` field identifies the existing row, if any.
   */
  private updateTimestamp(timestamp: any) {
    const timestripModel: JSONModel = super.getViewJSONModel(TimekeepingController.TIMESTRIP_MODEL);
    const existingTimestamps: any[] = timestripModel.getData() || [];
    const timestampIndex: number = existingTimestamps.findIndex((t: any) => t.time === timestamp.time);
    if (timestampIndex >= 0) {
      existingTimestamps[timestampIndex] = timestamp;
    } else {
      existingTimestamps.unshift(timestamp);
    }
    timestripModel.setData(existingTimestamps);
  }

  /**
   * Removes the local timestamp matching the supplied time key without waiting for backend acknowledgement.
   *
   * @param timestamp - Timestamp object whose `time` field identifies the local row to remove.
   */
  private deleteTimestamp(timestamp: any) {
    const timestripModel: JSONModel = super.getViewJSONModel(TimekeepingController.TIMESTRIP_MODEL);
    const existingTimestamps: any[] = timestripModel.getData() || [];
    const timestampIndex: number = existingTimestamps.findIndex((t: any) => t.time === timestamp.time);
    if (timestampIndex >= 0) {
      existingTimestamps.splice(timestampIndex, 1);
    }
    timestripModel.setData(existingTimestamps);
  }

  /** Closes the current socket and clears the controller's reference to it. */
  private disconnect() {
    if (this.socket) {
      console.debug("Disconnecting Timekeeping WebSocket ...");
      this.socket.close();
      delete this.socket;
    }
  }

  /**
   * Sends a JSON-encoded protocol command only when the socket is open; disconnected commands are discarded.
   *
   * @param command - Protocol object containing a command name and its payload, such as `AddStart` or `GetTimestrip`.
   */
  private sendCommand(command: any) {
    if (this.socket?.readyState === WebSocket.OPEN) {
      this.socket.send(JSON.stringify(command));
    }
  }

  /** Requests a full timestrip snapshot rather than an incremental timestamp update. */
  private sendGetTimestripCommand() {
    this.sendCommand({ GetTimestrip: null });
  }

  /** Adds a start timestamp using the browser's current clock, serialized as a UTC ISO string. */
  private sendAddStartCommand() {
    this.sendCommand({ AddStart: { time: new Date().toISOString() } });
  }

  /** Adds a finish timestamp using the browser's current clock, serialized as a UTC ISO string. */
  private sendAddFinishCommand() {
    this.sendCommand({ AddFinish: { time: new Date().toISOString() } });
  }

  private sendGetHeatsReadyToStartCommand() {
    this.sendCommand({ GetHeatsReadyToStart: null });
  }

  /**
   * Maps Space to a finish and `+` to a start, suppressing default browser handling for those keys.
   *
   * @param event - Browser keyboard event used to choose the timestamp command and stop further key handling.
   */
  private onKeyDown(event: KeyboardEvent): void {
    switch (event.key) {
      case " ":
        this.sendAddFinishCommand();
        event.stopPropagation();
        event.preventDefault();
        event.stopImmediatePropagation();
        break;
      case "+":
        this.sendAddStartCommand();
        event.stopPropagation();
        event.preventDefault();
        event.stopImmediatePropagation();
        break;
      default:
        break;
    }
  }
}
