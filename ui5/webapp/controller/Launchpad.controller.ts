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
import { Routes } from "../model/Routes";
import BaseController from "./Base.controller";

/** Refresh cadence for visible launchpad notifications while the controller is alive. */
const NOTIFICATIONS_POLL_INTERVAL_MS = 60_000;

/**
 * Launchpad navigation, session login/logout, and periodically refreshed visible notifications.
 *
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class LaunchpadController extends BaseController {
  readonly formatter: Formatter = Formatter;
  private readonly credentialsModel: JSONModel = new JSONModel({ username: "", password: "" });
  private readonly notificationsModel: JSONModel = new JSONModel([]);
  private notificationsTimer?: number;
  private popover?: ResponsivePopover;
  private popoverPromise?: Promise<ResponsivePopover>;

  /** Registers credential/notification models, checks the session identity, and starts notification polling. */
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

  /** Stops notification polling when the view's controller is destroyed. */
  onExit(): void {
    if (this.notificationsTimer) {
      globalThis.clearInterval(this.notificationsTimer);
      delete this.notificationsTimer;
    }
  }

  onNavToScoring(): void {
    super.getRouter().navTo(Routes.Scoring, {}, false /* history */);
  }

  onNavToStatistics(): void {
    super.getRouter().navTo(Routes.Statistics, {}, false /* history */);
  }

  onNavToMonitoring(): void {
    super.getRouter().navTo(Routes.Monitoring, {}, false /* history */);
  }

  onNavToSchedule(): void {
    super.getRouter().navTo(Routes.Schedule, {}, false /* history */);
  }

  onNavToTimekeeping(): void {
    super.getRouter().navTo(Routes.Timekeeping, {}, false /* history */);
  }

  onNavToProblems(): void {
    super.getRouter().navTo(Routes.Problems, {}, false /* history */);
  }

  /**
   * Moves keyboard focus from the username field to the password field.
   *
   * @param _event - Username submit event; unused because the target field is fixed.
   */
  onUserSubmit(_event: Input$SubmitEvent): void {
    super.byId("password")?.focus();
  }

  /**
   * Moves focus to the login button and submits the entered credentials.
   *
   * @param _event - Password submit event; unused because credentials are read from the model.
   */
  onPasswordSubmit(_event: Input$SubmitEvent): void {
    super.byId("login")?.focus();
    // perform login if return is pressed in password input field
    this.performLogin();
  }

  /**
   * Closes the login popover and submits the entered credentials.
   *
   * @param _event - Login-button press event; unused because credentials are read from the model.
   */
  onLoginPress(_event: Button$PressEvent): void {
    // close login popover when login button is pressed
    this.performLogin();
  }

  /**
   * Removes the displayed notification immediately and posts its read acknowledgement to the backend.
   *
   * @param event - Close event whose source notification item stores the backend notification ID in its counter.
   */
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

  /**
   * Refreshes visible notifications for the active regatta, logging failures without rejecting the polling task.
   *
   * @returns A promise resolving after the model loads or a lookup/request failure is logged.
   */
  private async loadNotifications(): Promise<void> {
    try {
      const regatta = await super.getActiveRegatta();
      await this.notificationsModel.loadData(`/api/regattas/${regatta.id}/visible_notifications`);
    } catch (err: unknown) {
      Log.error("Failed to load notifications", err as Error);
    }
  }

  /** Closes the login popover before submitting the entered credentials. */
  private performLogin() {
    if (this.popover) {
      this.popover.close();
      delete this.popover;
    }
    this.login();
  }

  /**
   * Logs out an authenticated user or toggles the login popover for an anonymous user.
   * Fragment loading is shared between callers and retried after failure.
   *
   * @param event - User-button press event whose source anchors the login popover when it opens.
   */
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
        id: this.getView()?.getId(),
        name: "de.regatta_hd.infoportal.view.LoginPopover",
        controller: this
      }).then((popover: any) => {
        // ... and initialize
        super.getView()?.addDependent(popover);
        popover.addStyleClass(super.getContentDensityClass());
        return popover;
      });

      // finish loading of fragment and open it. We swallow rejections explicitly
      // (logging only) — without a `.catch` an unhandled promise rejection would
      // surface in the browser console for every fragment-load failure.
      this.popoverPromise.then(
        (popover: ResponsivePopover) => {
          this.popover = popover;
          popover.openBy(eventSource);
        },
        (err: unknown) => {
          delete this.popoverPromise;
          Log.error("Failed to load login popover fragment", err as Error);
          super.showErrorMessageToast(super.i18n("msg.loginFailed"));
        }
      );
    }
  }

  /** Submits credentials, updates session identity on success, and clears the password model immediately. */
  private login(): void {
    const credentials: any = this.credentialsModel.getData();

    // see: https://api.jquery.com/jquery.ajax/
    $.ajax({
      type: "POST",
      url: "/api/login",
      data: JSON.stringify(credentials),
      contentType: "application/json",
      success: (result: { username: string; scope: string }) => {
        this.updateIdentity(true, result.username, result.scope);
        MessageToast.show(super.i18n("msg.loginSucceeded"));
        $(".sapMMessageToast").addClass("sapMMessageToastSuccess");
      },
      error: () => {
        MessageToast.show(super.i18n("msg.loginFailed"));
        $(".sapMMessageToast").addClass("sapMMessageToastDanger");
      }
    });

    // reset password
    this.credentialsModel.setProperty("/password", "");
  }

  /** Requests logout and clears the displayed identity after the backend confirms success. */
  private logout(): void {
    $.ajax({
      type: "POST",
      url: "/api/logout",
      success: () => {
        this.updateIdentity(false, "", "");
      }
    });
  }

  /** Restores the backend session identity, treating a failed identity request as anonymous. */
  private getIdentity(): void {
    $.ajax({
      type: "GET",
      url: "/api/identity",
      contentType: "application/json",
      success: (result: { username: string; scope: string }) => {
        this.updateIdentity(true, result.username, result.scope);
      },
      error: () => {
        this.updateIdentity(false, "", "");
      }
    });
  }

  /**
   * Updates authentication, username, and scope fields used by launchpad bindings.
   *
   * @param authenticated - Whether the backend session is authenticated.
   * @param name - Session username, or an empty string when clearing the displayed identity.
   * @param scope - Backend-provided authorization scope, or an empty string for an anonymous identity.
   */
  private updateIdentity(authenticated: boolean, name: string, scope: string): void {
    const identityModel: JSONModel = super.getComponentJSONModel("identity");
    identityModel.setProperty("/authenticated", authenticated);
    identityModel.setProperty("/username", name);
    identityModel.setProperty("/scope", scope);
  }

  /**
   * Reads the session flag used to choose between login and logout actions.
   *
   * @returns The identity model's authentication flag; it may be unset if the model is unavailable.
   */
  private isAuthenticated(): boolean {
    return this.getViewJSONModel("identity")?.getProperty("/authenticated");
  }
}
