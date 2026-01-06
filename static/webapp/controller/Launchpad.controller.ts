import * as $ from "jquery";
import { Button$PressEvent } from "sap/m/Button";
import { Input$SubmitEvent } from "sap/m/Input";
import MessageToast from "sap/m/MessageToast";
import ResponsivePopover from "sap/m/ResponsivePopover";
import Control from "sap/ui/core/Control";
import Fragment from "sap/ui/core/Fragment";
import JSONModel from "sap/ui/model/json/JSONModel";
import BaseController from "./Base.controller";
import Formatter from "../model/Formatter";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class LaunchpadController extends BaseController {

  readonly formatter: Formatter = Formatter;
  private readonly credentialsModel: JSONModel = new JSONModel({ username: "", password: "" });
  private popover?: ResponsivePopover;
  private popoverPromise?: Promise<ResponsivePopover>;

  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.setViewModel(this.credentialsModel, "credentials");
    super.getComponentJSONModel("messages");
    this.getIdentity();
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

  onNavToTimestrip(): void {
    super.getRouter().navTo("timestrip", {}, false /* history */);
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

      // finish loading of fragment and open it
      this.popoverPromise.then((popover: ResponsivePopover) => {
        this.popover = popover;
        popover.openBy(eventSource);
      });
    }
  }

  private login(): void {
    const credentials: any = this.credentialsModel.getData();

    // see: https://api.jquery.com/jquery.ajax/
    $.ajax({
      type: "POST",
      data: JSON.stringify(credentials),
      url: "/api/login",
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
