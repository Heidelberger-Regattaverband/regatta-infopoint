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

/**
 * Notification administration with create/edit dialogs, deletion, priority changes, and visibility updates.
 *
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class NotificationsController extends BaseController {
  readonly formatter: Formatter = Formatter;
  private readonly notificationsModel: JSONModel = new JSONModel();
  private readonly dialogModel: JSONModel = new JSONModel();
  private currentNotificationId?: number;
  private isEditMode: boolean = false;

  /** Registers the notification and dialog models and starts the initial administrative list request. */
  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.setViewModel(this.notificationsModel, MODEL_NOTIFICATIONS);
    super.setViewModel(this.dialogModel, "dialog");
    void this.loadNotifications();
  }

  navBack(): void {
    super.navBack(Routes.StartPage);
  }

  onRefresh(): void {
    void this.loadNotifications();
  }

  /** Opens a blank creation form without modifying an existing notification. */
  onAddNotification(): void {
    this.isEditMode = false;
    this.currentNotificationId = undefined;
    this.dialogModel.setData({
      title: "",
      text: "",
      priority: 0,
      visible: true
    });
    this.openDialog();
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
      this.isEditMode = true;
      this.currentNotificationId = notification.id;
      this.dialogModel.setData({
        title: notification.title,
        text: notification.text,
        priority: Number.parseInt(notification.priority || "0", 10),
        visible: notification.visible !== false
      });
      this.openDialog();
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
          if (action === MessageBox.Action.OK) {
            this.deleteNotification(notification.id);
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
      this.updateNotificationVisibility(notification.id, newState);
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
      this.updateNotification(notification.id, { ...notification, priority: newPriority });
    }
  }

  /** Requires a nonblank title, then dispatches creation or update according to the dialog's edit mode. */
  onSaveNotification(): void {
    const dialogData: any = this.dialogModel.getData();

    // Validation
    if (!dialogData.title || dialogData.title.trim() === "") {
      MessageToast.show(super.i18n("notifications.validation.titleRequired"));
      $(".sapMMessageToast").addClass("sapMMessageToastDanger");
      return;
    }

    if (this.isEditMode && this.currentNotificationId) {
      this.updateNotification(this.currentNotificationId, dialogData);
    } else {
      void this.createNotification(dialogData);
    }
  }

  onCancelDialog(): void {
    this.closeDialog();
  }

  /**
   * Starts a request for all active-regatta notifications, including invisible ones.
   * The promise covers regatta lookup and AJAX dispatch only; the list updates in the success callback.
   *
   * @returns A promise resolving after dispatch, or after a regatta-lookup failure is handled.
   * It does not wait for the GET request to finish.
   */
  private async loadNotifications(): Promise<void> {
    try {
      const regatta = await super.getActiveRegatta();
      const regattaId = regatta.id;

      // Load all notifications for admin (including invisible ones)
      const url = `/api/regattas/${regattaId}/notifications`;

      $.ajax({
        type: "GET",
        url: url,
        success: (notifications: any[]) => {
          this.notificationsModel.setData(notifications);
        },
        error: (xhr: any) => {
          console.error("Failed to load notifications:", xhr);
          MessageToast.show(super.i18n("notifications.error.loadFailed"));
          $(".sapMMessageToast").addClass("sapMMessageToastDanger");
        }
      });
    } catch (error) {
      console.error("Failed to get active regatta:", error);
      MessageToast.show(super.i18n("notifications.error.regattaFailed"));
      $(".sapMMessageToast").addClass("sapMMessageToastDanger");
    }
  }

  /**
   * Starts notification creation for the active regatta and closes the dialog on successful completion.
   * The promise covers regatta lookup and AJAX dispatch only, not completion of the POST request.
   *
   * @param data - Dialog fields for title, text, priority, and visibility.
   * @returns A promise resolving after dispatch, or after a regatta-lookup failure is handled.
   * It does not wait for the POST request to finish.
   */
  private async createNotification(data: any): Promise<void> {
    try {
      const regatta = await super.getActiveRegatta();
      const regattaId = regatta.id;

      $.ajax({
        type: "POST",
        url: `/api/regattas/${regattaId}/notifications`,
        contentType: "application/json",
        data: JSON.stringify({
          title: data.title,
          text: data.text,
          priority: Number.parseInt(data.priority, 10),
          visible: data.visible
        }),
        success: () => {
          MessageToast.show(super.i18n("notifications.success.created"));
          $(".sapMMessageToast").addClass("sapMMessageToastSuccess");
          void this.loadNotifications();
          this.closeDialog();
        },
        error: (xhr: any) => {
          console.error("Failed to create notification:", xhr);
          MessageToast.show(super.i18n("notifications.error.createFailed"));
          $(".sapMMessageToast").addClass("sapMMessageToastDanger");
        }
      });
    } catch (error) {
      console.error("Failed to get active regatta:", error);
      MessageToast.show(super.i18n("notifications.error.regattaFailed"));
      $(".sapMMessageToast").addClass("sapMMessageToastDanger");
    }
  }

  /**
   * Submits editable fields, then refreshes the list and closes the dialog after a successful update.
   *
   * @param notificationId - Backend ID of the notification to update.
   * @param data - Replacement title, text, priority, and visibility fields; priority is parsed as a decimal integer.
   */
  private updateNotification(notificationId: number, data: any): void {
    $.ajax({
      type: "PUT",
      url: `/api/notifications/${notificationId}`,
      contentType: "application/json",
      data: JSON.stringify({
        title: data.title,
        text: data.text,
        priority: Number.parseInt(data.priority, 10),
        visible: data.visible
      }),
      success: () => {
        MessageToast.show(super.i18n("notifications.success.updated"));
        $(".sapMMessageToast").addClass("sapMMessageToastSuccess");
        void this.loadNotifications();
        this.closeDialog();
      },
      error: (xhr: any) => {
        console.error("Failed to update notification:", xhr);
        MessageToast.show(super.i18n("notifications.error.updateFailed"));
        $(".sapMMessageToast").addClass("sapMMessageToastDanger");
      }
    });
  }

  /**
   * Updates only visibility and reloads the list on success or failure to reconcile the displayed switch state.
   *
   * @param notificationId - Backend ID of the notification whose visibility is changing.
   * @param visible - Whether the notification should be publicly visible.
   */
  private updateNotificationVisibility(notificationId: number, visible: boolean): void {
    $.ajax({
      type: "PUT",
      url: `/api/notifications/${notificationId}`,
      contentType: "application/json",
      data: JSON.stringify({
        visible: visible
      }),
      success: () => {
        MessageToast.show(super.i18n("notifications.success.visibilityUpdated"));
        $(".sapMMessageToast").addClass("sapMMessageToastSuccess");
        void this.loadNotifications();
      },
      error: (xhr: any) => {
        console.error("Failed to update notification visibility:", xhr);
        MessageToast.show(super.i18n("notifications.error.visibilityFailed"));
        $(".sapMMessageToast").addClass("sapMMessageToastDanger");
        void this.loadNotifications().catch((revertError: unknown) => {
          console.error("Failed to revert switch state:", revertError);
        });
      }
    });
  }

  /**
   * Deletes a previously confirmed notification and refreshes the list after backend success.
   *
   * @param notificationId - Backend ID of the notification to delete.
   */
  private deleteNotification(notificationId: number): void {
    $.ajax({
      type: "DELETE",
      url: `/api/notifications/${notificationId}`,
      success: () => {
        MessageToast.show(super.i18n("notifications.success.deleted"));
        $(".sapMMessageToast").addClass("sapMMessageToastSuccess");
        void this.loadNotifications();
      },
      error: (xhr: any) => {
        console.error("Failed to delete notification:", xhr);
        MessageToast.show(super.i18n("notifications.error.deleteFailed"));
        $(".sapMMessageToast").addClass("sapMMessageToastDanger");
      }
    });
  }

  private openDialog(): void {
    const dialog: Dialog = super.byId("notificationDialog") as Dialog;
    dialog.open();
  }

  private closeDialog(): void {
    const dialog: Dialog = super.byId("notificationDialog") as Dialog;
    dialog.close();
  }
}
