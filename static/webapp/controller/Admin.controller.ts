import * as $ from "jquery";
import { Button$PressEvent } from "sap/m/Button";
import Dialog from "sap/m/Dialog";
import MessageToast from "sap/m/MessageToast";
import { Switch$ChangeEvent } from "sap/m/Switch";
import Context from "sap/ui/model/Context";
import JSONModel from "sap/ui/model/json/JSONModel";
import Formatter from "../model/Formatter";
import BaseController from "./Base.controller";
import { ComboBox$SelectionChangeEvent } from "sap/m/ComboBox";

const MODEL_NOTIFICATIONS: string = "notifications";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class AdminController extends BaseController {

  readonly formatter: Formatter = Formatter;
  private readonly notificationsModel: JSONModel = new JSONModel();
  private readonly dialogModel: JSONModel = new JSONModel();
  private currentNotificationId?: number;
  private isEditMode: boolean = false;

  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.setViewModel(this.notificationsModel, MODEL_NOTIFICATIONS);
    super.setViewModel(this.dialogModel, "dialog");
    this.loadNotifications();
  }

  onRefresh(): void {
    this.loadNotifications();
  }

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

  onDeleteNotification(event: Button$PressEvent): void {
    const context: Context | undefined | null = event.getSource().getBindingContext(MODEL_NOTIFICATIONS);
    const notification: any = context?.getObject();

    if (notification) {
      this.currentNotificationId = notification.id;
      const deleteDialog: Dialog = super.byId("deleteDialog") as Dialog;
      deleteDialog.open();
    }
  }

  onVisibilityChange(event: Switch$ChangeEvent): void {
    const context: Context | undefined | null = event.getSource().getBindingContext(MODEL_NOTIFICATIONS);
    const notification: any = context?.getObject();
    const newState: boolean = event.getParameters().state || false;

    if (notification) {
      this.updateNotificationVisibility(notification.id, newState);
    }
  }

  onPriorityChange(event: ComboBox$SelectionChangeEvent): void {
    const context: Context | undefined | null = event.getSource().getBindingContext(MODEL_NOTIFICATIONS);
    const notification: any = context?.getObject();
    const newPriority: number = Number.parseInt(event.getParameters().selectedItem?.getKey() || "0", 10);

    if (notification) {
      this.updateNotification(notification.id, { ...notification, priority: newPriority });
    }
  }

  onSaveNotification(): void {
    const dialogData: any = this.dialogModel.getData();

    // Validation
    if (!dialogData.title || dialogData.title.trim() === "") {
      MessageToast.show(super.i18n("admin.validation.titleRequired"));
      $(".sapMMessageToast").addClass("sapMMessageToastDanger");
      return;
    }

    if (this.isEditMode && this.currentNotificationId) {
      this.updateNotification(this.currentNotificationId, dialogData);
    } else {
      this.createNotification(dialogData);
    }
  }

  onCancelDialog(): void {
    this.closeDialog();
  }

  onConfirmDelete(): void {
    if (this.currentNotificationId) {
      this.deleteNotification(this.currentNotificationId);
    }
    this.closeDeleteDialog();
  }

  onCancelDelete(): void {
    this.closeDeleteDialog();
  }

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
          MessageToast.show(super.i18n("admin.error.loadFailed"));
          $(".sapMMessageToast").addClass("sapMMessageToastDanger");
        }
      });
    } catch (error) {
      console.error("Failed to get active regatta:", error);
      MessageToast.show(super.i18n("admin.error.regattaFailed"));
      $(".sapMMessageToast").addClass("sapMMessageToastDanger");
    }
  }

  private createNotification(data: any): void {
    this.getActiveRegatta().then((regatta) => {
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
        success: (result: any) => {
          MessageToast.show(super.i18n("admin.success.created"));
          $(".sapMMessageToast").addClass("sapMMessageToastSuccess");
          this.loadNotifications();
          this.closeDialog();
        },
        error: (xhr: any) => {
          console.error("Failed to create notification:", xhr);
          MessageToast.show(super.i18n("admin.error.createFailed"));
          $(".sapMMessageToast").addClass("sapMMessageToastDanger");
        }
      });
    });
  }

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
      success: (result: any) => {
        MessageToast.show(super.i18n("admin.success.updated"));
        $(".sapMMessageToast").addClass("sapMMessageToastSuccess");
        this.loadNotifications();
        this.closeDialog();
      },
      error: (xhr: any) => {
        console.error("Failed to update notification:", xhr);
        MessageToast.show(super.i18n("admin.error.updateFailed"));
        $(".sapMMessageToast").addClass("sapMMessageToastDanger");
      }
    });
  }

  private updateNotificationVisibility(notificationId: number, visible: boolean): void {
    $.ajax({
      type: "PUT",
      url: `/api/notifications/${notificationId}`,
      contentType: "application/json",
      data: JSON.stringify({
        visible: visible
      }),
      success: (result: any) => {
        MessageToast.show(super.i18n("admin.success.visibilityUpdated"));
        $(".sapMMessageToast").addClass("sapMMessageToastSuccess");
        this.loadNotifications();
      },
      error: (xhr: any) => {
        console.error("Failed to update notification visibility:", xhr);
        MessageToast.show(super.i18n("admin.error.visibilityFailed"));
        $(".sapMMessageToast").addClass("sapMMessageToastDanger");
        // Revert the switch state
        this.loadNotifications();
      }
    });
  }

  private deleteNotification(notificationId: number): void {
    $.ajax({
      type: "DELETE",
      url: `/api/notifications/${notificationId}`,
      success: (result: any) => {
        MessageToast.show(super.i18n("admin.success.deleted"));
        $(".sapMMessageToast").addClass("sapMMessageToastSuccess");
        this.loadNotifications();
      },
      error: (xhr: any) => {
        console.error("Failed to delete notification:", xhr);
        MessageToast.show(super.i18n("admin.error.deleteFailed"));
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

  private closeDeleteDialog(): void {
    const deleteDialog: Dialog = super.byId("deleteDialog") as Dialog;
    deleteDialog.close();
  }

  navBack(): void {
    super.navBack("startpage");
  }
}