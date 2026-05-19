import * as $ from "jquery";
import Log from "sap/base/Log";
import { Button$PressEvent } from "sap/m/Button";
import { Input$SubmitEvent } from "sap/m/Input";
import NotificationList from "sap/m/NotificationList";
import NotificationListItem from "sap/m/NotificationListItem";
import ResponsivePopover from "sap/m/ResponsivePopover";
import Event from "sap/ui/base/Event";
import Control from "sap/ui/core/Control";
import Fragment from "sap/ui/core/Fragment";
import JSONModel from "sap/ui/model/json/JSONModel";
import Formatter from "../model/Formatter";
import BaseController from "./Base.controller";

interface Identity {
  username: string;
  scope: string;
}

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class LaunchpadController extends BaseController {

  private static readonly EMPTY_CREDENTIALS: { username: string; password: string } = { username: "", password: "" };

  readonly formatter: Formatter = Formatter;
  private readonly credentialsModel: JSONModel = new JSONModel({ ...LaunchpadController.EMPTY_CREDENTIALS });
  private popover?: ResponsivePopover;
  private popoverPromise?: Promise<ResponsivePopover>;

  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.setViewModel(this.credentialsModel, "credentials");
    // Best-effort identity check; failures (e.g. 401) silently leave the UI in
    // the unauthenticated state. We intentionally do not surface an error toast
    // here — an anonymous visit is the default.
    void this.fetchIdentity();
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

  onUserSubmit(_event: Input$SubmitEvent): void {
    super.byId("password")?.focus();
  }

  onPasswordSubmit(_event: Input$SubmitEvent): void {
    // Submit the form; do not shift focus to the login button — the popover is
    // about to close anyway, and the extra focus call triggers a redundant render.
    void this.performLogin();
  }

  onLoginPress(_event: Button$PressEvent): void {
    void this.performLogin();
  }

  /**
   * Called by the popover's `afterClose` event. Wipes any credentials still
   * sitting in the view model so the password does not linger in memory after
   * the popover (which lives as long as the view) has been dismissed.
   */
  onLoginPopoverAfterClose(_event: Event): void {
    this.clearCredentials();
  }

  onNotificationClose(event: Event): void {
    const item: NotificationListItem = event.getSource();
    (item.getParent() as NotificationList).removeItem(item);
    const notificationId: number = item.getCounter();

    $.ajax({
      type: "POST",
      url: `/api/notifications/${notificationId}/read`,
      success: () => {
        // refresh notifications model
        super.getComponentJSONModel("notifications")?.refresh();
      }
    });
  }

  /**
   * Performs the login flow:
   *  1. closes the popover (so the user gets immediate feedback),
   *  2. awaits the login HTTP call,
   *  3. updates the identity model only on success,
   *  4. wipes credentials from the view model regardless of outcome.
   *
   * Returns a Promise so callers (and tests) can await the outcome.
   */
  private async performLogin(): Promise<void> {
    if (this.popover) {
      this.popover.close();
      delete this.popover;
    }
    try {
      await this.login();
    } finally {
      // Always clear credentials, even on failure, so an attacker with later
      // DOM access cannot recover them from the view model.
      this.clearCredentials();
    }
  }

  onShowLoginPress(event: Button$PressEvent): void {
    const eventSource: Control = event.getSource();

    if (this.isAuthenticated()) {
      void this.logout();
      return;
    }

    if (this.popover?.isOpen()) {
      // close login dialog if it's already open
      this.popover.close();
      delete this.popover;
      return;
    }

    // check if fragment is already loaded or not
    // load fragment ...
    this.popoverPromise ??= Fragment.load({
      id: this.getView()?.getId(), name: "de.regatta_hd.infoportal.view.LoginPopover", controller: this
    }).then((popover: Control | Control[]) => {
      const responsivePopover: ResponsivePopover = (Array.isArray(popover) ? popover[0] : popover) as ResponsivePopover;
      // ... and initialize
      super.getView()?.addDependent(responsivePopover);
      responsivePopover.addStyleClass(super.getContentDensityClass());
      return responsivePopover;
    });

    // finish loading of fragment and open it. We swallow rejections explicitly
    // (logging only) — without a `.catch` an unhandled promise rejection would
    // surface in the browser console for every fragment-load failure.
    this.popoverPromise.then((popover: ResponsivePopover) => {
      this.popover = popover;
      popover.openBy(eventSource);
    }, (err: unknown) => {
      // Reset the cached promise so a future click can retry.
      delete this.popoverPromise;
      Log.error("Failed to load login popover fragment", err as Error);
      super.showErrorMessageToast(super.i18n("msg.loginFailed"));
    });
  }

  /**
   * POSTs the credentials and resolves once the response has been processed.
   * Identity is updated only on a successful 2xx response; on failure the
   * caller sees a rejection and the identity model is left unchanged.
   */
  private login(): Promise<void> {
    const credentials: { username: string; password: string } = this.credentialsModel.getData();

    return new Promise<void>((resolve, reject) => {
      // see: https://api.jquery.com/jquery.ajax/
      $.ajax({
        type: "POST",
        data: JSON.stringify(credentials),
        url: "/api/login",
        contentType: "application/json",
        success: (result: Identity) => {
          this.updateIdentity(true, result.username, result.scope);
          super.showInfoMessageToast(super.i18n("msg.loginSucceeded"));
          resolve();
        },
        error: (xhr: JQuery.jqXHR) => {
          Log.warning(`Login failed: ${xhr.status} ${xhr.statusText}`);
          super.showErrorMessageToast(super.i18n("msg.loginFailed"));
          reject(new Error(`Login failed: ${xhr.status}`));
        }
      });
    });
  }

  private logout(): Promise<void> {
    return new Promise<void>((resolve) => {
      $.ajax({
        type: "POST",
        url: "/api/logout",
        complete: () => {
          // Always revert to anonymous state — even on transport errors the
          // user explicitly asked to log out, so we do not keep credentials.
          this.updateIdentity(false, "", "");
          this.clearCredentials();
          resolve();
        }
      });
    });
  }

  /**
   * Probes `/api/identity` to determine whether the browser still has a valid
   * session. A 401 (or any error) is treated as "not authenticated" without
   * raising a user-visible toast, because an anonymous user is the default.
   */
  private fetchIdentity(): Promise<void> {
    return new Promise<void>((resolve) => {
      $.ajax({
        type: "GET",
        url: "/api/identity",
        contentType: "application/json",
        success: (result: Identity) => {
          this.updateIdentity(true, result.username, result.scope);
          resolve();
        },
        error: () => {
          this.updateIdentity(false, "", "");
          resolve();
        }
      });
    });
  }

  /**
   * Resets both username and password fields. Called after every login attempt
   * (success or failure), after logout, and via the popover's `afterClose`
   * hook so credentials never linger in the view model.
   */
  private clearCredentials(): void {
    this.credentialsModel.setData({ ...LaunchpadController.EMPTY_CREDENTIALS });
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