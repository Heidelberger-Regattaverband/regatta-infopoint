import JSONModel from "sap/ui/model/json/JSONModel";
import BaseController from "./Base.controller";
import MessageToast from "sap/m/MessageToast";
import ResponsivePopover from "sap/m/ResponsivePopover";
import Fragment from "sap/ui/core/Fragment";
import * as $ from "jquery";
import Control from "sap/ui/core/Control";
import { Button$PressEvent } from "sap/m/Button";
import { Input$SubmitEvent } from "sap/m/Input";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class Launchpad extends BaseController {

  private credentialsModel: JSONModel = new JSONModel({ username: "", password: "" });
  private popover?: ResponsivePopover;
  private popoverPromise?: Promise<ResponsivePopover>;

  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());
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

  onNavToMonitoring(): void {
    super.getRouter().navTo("monitoring", {}, false /* history */);
  }

  onNavToKiosk(): void {
    super.getRouter().navTo("kiosk", {}, false /* history */);
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
            super.getView()?.addDependent(popover);
            popover.addStyleClass(super.getContentDensityClass());
            return popover;
          });
        }

        // finish loading of fragment and open it
        this.popoverPromise.then((popover: ResponsivePopover) => {
          this.popover = popover;
          popover.openBy(eventSource);
        });
      }
    } else {
      this.logout();
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
      success: (result: { username: any; }) => {
        this.updateIdentity(true, result.username);
        this.credentialsModel.setProperty("/username", result.username);
        MessageToast.show(super.i18n("msg.loginSucceeded"));
        $(".sapMMessageToast").removeClass("sapMMessageToastDanger").addClass("sapMMessageToastSuccess");
      },
      error: (result: any) => {
        MessageToast.show(super.i18n("msg.loginFailed"));
        $(".sapMMessageToast").removeClass("sapMMessageToastSuccess").addClass("sapMMessageToastDanger");
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
        this.updateIdentity(false, "");
      }
    });
  }

  private getIdentity(): void {
    $.ajax({
      type: "GET",
      url: "/api/identity",
      contentType: "application/json",
      success: (result: { username: string; }) => {
        this.updateIdentity(true, result.username);
        this.credentialsModel.setProperty("/username", result.username);
      },
      error: (result: any) => {
        this.updateIdentity(false, "");
      }
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
