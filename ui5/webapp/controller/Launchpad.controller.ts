import $ from "jquery";
import Log from "sap/base/Log";
import { Button$PressEvent } from "sap/m/Button";
import { Input$SubmitEvent } from "sap/m/Input";
import MessageToast from "sap/m/MessageToast";
import NotificationList from "sap/m/NotificationList";
import NotificationListItem from "sap/m/NotificationListItem";
import ResponsivePopover from "sap/m/ResponsivePopover";
import Event from "sap/ui/base/Event";
import Control from "sap/ui/core/Control";
import Fragment from "sap/ui/core/Fragment";
import JSONModel from "sap/ui/model/json/JSONModel";
import Formatter from "../model/Formatter";
import BaseController from "./Base.controller";

const NOTIFICATIONS_POLL_INTERVAL_MS = 60_000;

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class LaunchpadController extends BaseController {

  readonly formatter: Formatter = Formatter;
  private readonly credentialsModel: JSONModel = new JSONModel({ username: "", password: "" });
  private readonly notificationsModel: JSONModel = new JSONModel([]);
  private notificationsTimer?: number;
  private popover?: ResponsivePopover;
  private popoverPromise?: Promise<ResponsivePopover>;

  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.setViewModel(this.credentialsModel, "credentials");
    super.setViewModel(this.notificationsModel, "notifications");
    this.getIdentity();
    void this.loadNotifications();
    this.notificationsTimer = globalThis.setInterval(() => {
      void this.loadNotifications();
    }, NOTIFICATIONS_POLL_INTERVAL_MS);
  }

  onExit(): void {
    if (this.notificationsTimer) {
      globalThis.clearInterval(this.notificationsTimer);
      delete this.notificationsTimer;
    }
  }

  onNavToScoring(): void {
    super.getRouter().navTo("scoring", {}, false /* history */);
  }

  onNavToStatistics(): void {
    super.getRouter().navTo("statistics", {}, false /* history */);
  }

  onNavToMonitoring(): void {
    super.getRouter().navTo("monitoring", {}, false /* history */);
  }

  onNavToSchedule(): void {
    super.getRouter().navTo("schedule", {}, false /* history */);
  }

  onNavToTimekeeping(): void {
    super.getRouter().navTo("timekeeping", {}, false /* history */);
  }

  onNavToProblems(): void {
    super.getRouter().navTo("problems", {}, false /* history */);
  }

  onUserSubmit(event: Input$SubmitEvent): void {
    super.byId("password")?.focus();
  }

  onPasswordSubmit(event: Input$SubmitEvent): void {
    super.byId("login")?.focus();
    // perform login if return is pressed in password input field
    this.performLogin();
  }

  onLoginPress(event: Button$PressEvent): void {
    // close login popover when login button is pressed
    this.performLogin();
  }

  onNotificationClose(event: Event): void {
    const item: NotificationListItem = event.getSource();
    (item.getParent() as NotificationList).removeItem(item);
    const notificationId: number = item.getCounter();

    $.ajax({
      type: "POST",
      url: `/api/notifications/${notificationId}/read`,
      success: () => {
        void this.loadNotifications();
      }
    });
  }

  private async loadNotifications(): Promise<void> {
    try {
      const regatta = await super.getActiveRegatta();
      await this.notificationsModel.loadData(`/api/regattas/${regatta.id}/visible_notifications`);
    } catch (err: unknown) {
      Log.error("Failed to load notifications", err as Error);
    }
  }

  private performLogin() {
    if (this.popover) {
      this.popover.close();
      delete this.popover;
    }
    this.login();
  }

  onShowLoginPress(event: Button$PressEvent): void {
    const eventSource: Control = event.getSource();

    if (this.isAuthenticated()) {
      this.logout();
    } else if (this.popover?.isOpen()) {
      // close login dialog if it's already open
      this.popover.close();
      delete this.popover;
    } else {
      // check if fragment is already loaded or not
      // load fragment ...
      this.popoverPromise ??= Fragment.load({
        id: this.getView()?.getId(), name: "de.regatta_hd.infoportal.view.LoginPopover", controller: this
      }).then((popover: any) => {
        // ... and initialize
        super.getView()?.addDependent(popover);
        popover.addStyleClass(super.getContentDensityClass());
        return popover;
      });

      // finish loading of fragment and open it. We swallow rejections explicitly
      // (logging only) — without a `.catch` an unhandled promise rejection would
      // surface in the browser console for every fragment-load failure.
      this.popoverPromise.then((popover: ResponsivePopover) => {
        this.popover = popover;
        popover.openBy(eventSource);
      }, (err: unknown) => {
        delete this.popoverPromise;
        Log.error("Failed to load login popover fragment", err as Error);
        super.showErrorMessageToast(super.i18n("msg.loginFailed"));
      });
    }
  }

  private login(): void {
    const credentials: any = this.credentialsModel.getData();

    // see: https://api.jquery.com/jquery.ajax/
    $.ajax({
      type: "POST",
      url: "/api/login",
      data: JSON.stringify(credentials),
      contentType: "application/json",
      success: (result: { username: string, scope: string }) => {
        this.updateIdentity(true, result.username, result.scope);
        MessageToast.show(super.i18n("msg.loginSucceeded"));
        $(".sapMMessageToast").addClass("sapMMessageToastSuccess");
      },
      error: (result: any) => {
        MessageToast.show(super.i18n("msg.loginFailed"));
        $(".sapMMessageToast").addClass("sapMMessageToastDanger");
      }
    });

    // reset password
    this.credentialsModel.setProperty("/password", "");
  }

  private logout(): void {
    $.ajax({
      type: "POST",
      url: "/api/logout",
      success: (result: any) => {
        this.updateIdentity(false, "", "");
      }
    });
  }

  private getIdentity(): void {
    $.ajax({
      type: "GET",
      url: "/api/identity",
      contentType: "application/json",
      success: (result: { username: string, scope: string }) => {
        this.updateIdentity(true, result.username, result.scope);
      },
      error: (result: any) => {
        this.updateIdentity(false, "", "");
      }
    });
  }

  private updateIdentity(authenticated: boolean, name: string, scope: string): void {
    const identityModel: JSONModel = super.getComponentJSONModel("identity");
    identityModel.setProperty("/authenticated", authenticated);
    identityModel.setProperty("/username", name);
    identityModel.setProperty("/scope", scope);
  }

  private isAuthenticated(): boolean {
    return this.getViewJSONModel("identity")?.getProperty("/authenticated");
  }
}
