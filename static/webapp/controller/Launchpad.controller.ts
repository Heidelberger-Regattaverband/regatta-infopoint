import JSONModel from "sap/ui/model/json/JSONModel";
import BaseController from "./Base.controller";
import MyComponent from "de/regatta_hd/Component";
import Formatter from "../model/Formatter";
import Event from "sap/ui/base/Event";
import MessageToast from "sap/m/MessageToast";
import ResponsivePopover from "sap/m/ResponsivePopover";
import Fragment from "sap/ui/core/Fragment";
import * as $ from "jquery";
import Control from "sap/ui/core/Control";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class Launchpad extends BaseController {

  static formatter: Formatter;
  private credentialsModel: JSONModel;
  private popover?: ResponsivePopover;
  private popoverPromise?: Promise<ResponsivePopover>;

  onInit(): void {
    super.getView()?.addStyleClass((this.getOwnerComponent() as MyComponent).getContentDensityClass());

    this.credentialsModel = new JSONModel({ username: "", password: "" });
    super.setViewModel(this.credentialsModel, "credentials");
    this.getIdentity();
  }

  onNavToRaces(): void {
    super.getRouter().navTo("races", {}, false /* history */);
  }

  onNavToHeats(): void {
    super.getRouter().navTo("heats", {}, false /* history */);
  }

  onNavToParticipatingClubs(): void {
    super.getRouter().navTo("participatingClubs", {}, false /* history */);
  }

  onNavToScoring(): void {
    super.getRouter().navTo("scoring", {}, false /* history */);
  }

  onNavToStatistics(): void {
    super.getRouter().navTo("statistics", {}, false /* history */);
  }

  onNavToKiosk(): void {
    super.getRouter().navTo("kiosk", {}, false /* history */);
  }

  onUserSubmit(event: Event): void {
    super.byId("password")?.focus();
  }

  onPasswordSubmit(event: Event): void {
    super.byId("login")?.focus();
    // perform login if return is pressed in password input field
    this.onLoginPress(event);
  }

  onLoginPress(event: Event): void {
    // close login popover when login button is pressed
    if (this.popover) {
      this.popover.close();
      delete this.popover;
    }
    this.login();
  }

  onShowLoginPress(event: Event): void {
    const eventSource: Control = event.getSource();
    const that = this;

    if (!this.isAuthenticated()) {
      if (this.popover?.isOpen()) {
        // close login dialog if it's already open
        this.popover.close();
        delete this.popover;
      } else {
        // check if fragment is already loaded or not
        if (!this.popoverPromise) {
          // load fragment ...
          this.popoverPromise = Fragment.load({
            id: this.getView()?.getId(), name: "de.regatta_hd.infoportal.view.LoginPopover", controller: this
          }).then((popover: any) => {
            // ... and initialize
            that.getView()?.addDependent(popover);
            popover.addStyleClass((that.getOwnerComponent() as MyComponent).getContentDensityClass());
            return popover;
          });
        }

        // finish loading of fragment and open it
        this.popoverPromise.then((popover: ResponsivePopover) => {
          that.popover = popover;
          popover.openBy(eventSource);
        });
      }
    } else {
      this.logout();
    }
  }

  private login(): void {
    const credentials: any = this.credentialsModel.getData();
    const that = this;

    // see: https://api.jquery.com/jquery.ajax/
    $.ajax({
      type: "POST",
      data: JSON.stringify(credentials),
      url: "/api/login",
      contentType: "application/json",
      success: function (result: { username: any; }) {
        that.updateIdentity(true, result.username);
        that.credentialsModel.setProperty("/username", result.username);
        MessageToast.show(that.i18n("msg.loginSucceeded", undefined));
        $(".sapMMessageToast").removeClass("sapMMessageToastDanger").addClass("sapMMessageToastSuccess");
      }.bind(this),
      error: function (result: any) {
        MessageToast.show(that.i18n("msg.loginFailed", undefined));
        $(".sapMMessageToast").removeClass("sapMMessageToastSuccess").addClass("sapMMessageToastDanger");
      }.bind(this)
    });

    // reset password
    this.credentialsModel.setProperty("/password", "");
  }

  private logout(): void {
    const that = this;
    $.ajax({
      type: "POST",
      url: "/api/logout",
      success: function (result: any) {
        that.updateIdentity(false, "");
      }.bind(this)
    });
  }

  private getIdentity(): void {
    const that = this;
    $.ajax({
      type: "GET",
      url: "/api/identity",
      contentType: "application/json",
      success: function (result: { username: string; }) {
        that.updateIdentity(true, result.username);
        that.credentialsModel.setProperty("/username", result.username);
      }.bind(this),
      error: function (result: any) {
        that.updateIdentity(false, "");
      }.bind(this)
    });
  }

  private updateIdentity(authenticated: boolean, name: string): void {
    const identityModel: JSONModel = super.getComponentModel("identity") as JSONModel;
    identityModel.setProperty("/authenticated", authenticated);
    identityModel.setProperty("/username", name);
  }

  private isAuthenticated(): boolean {
    return this.getViewModel("identity")?.getProperty("/authenticated");
  }
}
