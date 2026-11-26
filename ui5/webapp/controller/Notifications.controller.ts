import $ from "jquery";
import { Button$PressEvent } from "sap/m/Button";
import { ComboBox$SelectionChangeEvent } from "sap/m/ComboBox";
import Dialog from "sap/m/Dialog";
import MessageBox from "sap/m/MessageBox";
import MessageToast from "sap/m/MessageToast";
import { Switch$ChangeEvent } from "sap/m/Switch";
import Context from "sap/ui/model/Context";
import JSONModel from "sap/ui/model/json/JSONModel";
import Formatter from "../model/Formatter";
import { Routes } from "../model/Routes";
import BaseController from "./Base.controller";

const MODEL_NOTIFICATIONS: string = "notifications";
const WRITE_ACTIONS = {
  create: { method: "POST", success: "notifications.success.created", error: "notifications.error.createFailed" },
  update: { method: "PUT", success: "notifications.success.updated", error: "notifications.error.updateFailed" },
  delete: { method: "DELETE", success: "notifications.success.deleted", error: "notifications.error.deleteFailed" },
  visibility: {
    method: "PUT",
    success: "notifications.success.visibilityUpdated",
    error: "notifications.error.visibilityFailed"
  }
} as const;

/**
 * Notification administration with create/edit dialogs, deletion, priority changes, and visibility updates.
 *
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class NotificationsController extends BaseController {
  readonly formatter: Formatter = Formatter;
  private readonly notificationsModel: JSONModel = new JSONModel([]);
  private readonly dialogModel: JSONModel = new JSONModel();
  private readonly stateModel: JSONModel = new JSONModel({ saving: false });
  private dialog: Dialog;

  /** Registers the notification and dialog models and reloads notifications whenever the route matches. */
  onInit(): void {
    super.onInit?.();
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.setViewModel(this.notificationsModel, MODEL_NOTIFICATIONS);
    super.setViewModel(this.dialogModel, "dialog");
    super.setViewModel(this.stateModel, "notificationState");
    this.dialog = super.byId("notificationDialog") as Dialog;
    this.dialog.setEscapeHandler((handlers) => {
      if (this.stateModel.getProperty("/saving")) {
        handlers.reject();
      } else {
        this.dialogModel.setData({});
        handlers.resolve();
      }
    });
    super.getRouter()?.getRoute(Routes.Notifications)?.attachMatched(this.onRefresh, this);
  }

  /** Invalidates pending reads and detaches the route listener on destruction. */
  onExit(): void {
    super.getRouter()?.getRoute(Routes.Notifications)?.detachMatched(this.onRefresh, this);
    super.onExit?.();
  }

  navBack(): void {
    super.navBack(Routes.StartPage);
  }

  onRefresh(): void {
    super.runAsync(this.loadNotifications(), "Failed to load notifications:");
  }

  /** Opens a blank creation form without modifying an existing notification. */
  onAddNotification(): void {
    this.openDialog({
      title: "",
      text: "",
      priority: 0,
      visible: true
    });
  }

  /**
   * Copies the selected notification into the dialog model so edits stay separate from the displayed list.
   *
   * @param event - Edit-button press event whose binding context identifies the notification to edit.
   */
  onEditNotification(event: Button$PressEvent): void {
    const context: Context | undefined | null = event.getSource().getBindingContext(MODEL_NOTIFICATIONS);
    const notification: any = context?.getObject();

    if (notification) {
      this.openDialog({
        id: notification.id,
        title: notification.title,
        text: notification.text,
        priority: Number.parseInt(notification.priority || "0", 10),
        visible: notification.visible !== false
      });
    }
  }

  /**
   * Deletes the selected notification only after the user confirms.
   *
   * @param event - Delete-button press event whose binding context identifies the notification to confirm.
   */
  onDeleteNotification(event: Button$PressEvent): void {
    const context: Context | undefined | null = event.getSource().getBindingContext(MODEL_NOTIFICATIONS);
    const notification: any = context?.getObject();

    if (notification) {
      MessageBox.confirm(super.i18n("notifications.delete.message"), {
        title: super.i18n("notifications.delete.title"),
        emphasizedAction: MessageBox.Action.CANCEL,
        onClose: (action: any) => {
          if (action === MessageBox.Action.OK && this.isViewAlive()) {
            super.runAsync(this.writeNotification("delete", notification.id), "Failed to delete notification:");
          }
        }
      });
    }
  }

  /**
   * Submits the notification's new visibility flag without changing its other editable fields.
   *
   * @param event - Switch-change event providing the notification binding context and requested visibility state.
   */
  onVisibilityChange(event: Switch$ChangeEvent): void {
    const context: Context | undefined | null = event.getSource().getBindingContext(MODEL_NOTIFICATIONS);
    const notification: any = context?.getObject();
    const newState: boolean = event.getParameters().state || false;

    if (notification) {
      super.runAsync(
        this.writeNotification("visibility", notification.id, { visible: newState }),
        "Failed to update notification visibility:"
      );
    }
  }

  /**
   * Sends the selected priority along with the notification's other editable fields.
   *
   * @param event - Combo-box selection event providing the notification context and numeric priority key.
   */
  onPriorityChange(event: ComboBox$SelectionChangeEvent): void {
    const context: Context | undefined | null = event.getSource().getBindingContext(MODEL_NOTIFICATIONS);
    const notification: any = context?.getObject();
    const newPriority: number = Number.parseInt(event.getParameters().selectedItem?.getKey() || "0", 10);

    if (notification) {
      super.runAsync(
        this.writeNotification("update", notification.id, { ...notification, priority: newPriority }),
        "Failed to update notification priority:"
      );
    }
  }

  /** Starts at most one dialog write; pending saves are also disabled through the state model. */
  onSaveNotification(): void {
    super.runAsync(this.saveNotification(), "Failed to save notification:");
  }

  /** Locks the form until the write settles; failures retain its input for retry. */
  private async saveNotification(): Promise<void> {
    const data: any = this.dialogModel.getData();
    if (
      this.stateModel.getProperty("/saving") ||
      !Object.hasOwn(data, "title") ||
      !this.dialog.isOpen() ||
      !this.isViewAlive()
    ) {
      return;
    }
    const dialogData: any = { ...data };

    // Validation
    if (!dialogData.title || dialogData.title.trim() === "") {
      this.showNotificationMessage("notifications.validation.titleRequired", false);
      return;
    }

    this.stateModel.setProperty("/saving", true);
    try {
      const succeeded = await this.writeNotification(
        dialogData.id === undefined ? "create" : "update",
        dialogData.id,
        dialogData
      );
      if (succeeded) {
        this.closeDialog();
      }
    } finally {
      if (this.isViewAlive()) {
        this.stateModel.setProperty("/saving", false);
      }
    }
  }

  onCancelDialog(): void {
    if (!this.stateModel.getProperty("/saving")) {
      this.closeDialog();
    }
  }

  /**
   * Waits for all active-regatta notifications, including invisible ones.
   * Only the newest request for a live view may update the list or report a failure.
   *
   * @returns Whether the request completed successfully and was still relevant when its data arrived.
   */
  private async loadNotifications(): Promise<boolean> {
    let errorKey = "notifications.error.regattaFailed";
    try {
      const regatta = await super.getActiveRegatta();
      if (!this.isViewAlive()) {
        return false;
      }
      errorKey = "notifications.error.loadFailed";
      const notifications: any[] = await $.ajax({ type: "GET", url: `/api/regattas/${regatta.id}/notifications` });
      if (!this.isViewAlive()) {
        return false;
      }
      this.notificationsModel.setData(notifications);
      return true;
    } catch (error) {
      if (this.isViewAlive()) {
        console.error("Failed to load notifications:", error);
        this.showNotificationMessage(errorKey, false);
      }
      return false;
    }
  }

  /** Waits for one write, then reconciles the list; dialog ownership is handled only by Save. */
  private async writeNotification(
    action: keyof typeof WRITE_ACTIONS,
    notificationId?: number,
    data?: any
  ): Promise<boolean> {
    if (!this.isViewAlive()) {
      return false;
    }
    const { method, success, error: failure } = WRITE_ACTIONS[action];
    const payload =
      data && action !== "visibility"
        ? { title: data.title, text: data.text, priority: Number.parseInt(data.priority, 10), visible: data.visible }
        : data;
    let errorKey = "notifications.error.regattaFailed";
    try {
      const url =
        action === "create"
          ? `/api/regattas/${(await super.getActiveRegatta()).id}/notifications`
          : `/api/notifications/${notificationId}`;
      if (!this.isViewAlive()) {
        return false;
      }
      errorKey = failure;
      await $.ajax({ type: method, url, contentType: "application/json", data: payload && JSON.stringify(payload) });
      if (!this.isViewAlive()) {
        return false;
      }
      this.showNotificationMessage(success, true);
      this.onRefresh();
      return true;
    } catch (error) {
      if (this.isViewAlive()) {
        console.error("Failed to modify notification:", error);
        this.showNotificationMessage(errorKey, false);
        if (action === "visibility") {
          this.onRefresh();
        }
      }
      return false;
    }
  }

  private isViewAlive(): boolean {
    return super.getView()?.isDestroyed() === false;
  }

  private showNotificationMessage(key: string, succeeded: boolean): void {
    MessageToast.show(super.i18n(key));
    $(".sapMMessageToast").addClass(succeeded ? "sapMMessageToastSuccess" : "sapMMessageToastDanger");
  }

  private openDialog(data: any): void {
    if (this.stateModel.getProperty("/saving")) {
      return;
    }
    this.dialogModel.setData(data);
    this.dialog.open();
  }

  private closeDialog(): void {
    if (this.isViewAlive() && this.dialog.isOpen()) {
      // Reject queued Save events even while UI5's closing animation still reports isOpen().
      this.dialogModel.setData({});
      this.dialog.close();
    }
  }
}
