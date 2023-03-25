sap.ui.define([
  'sap/ui/core/Fragment',
  "sap/ui/model/json/JSONModel",
  "sap/m/MessageToast",
  "de/regatta_hd/infopoint/controller/Base.controller",
  "../model/Formatter"
], function (Fragment, JSONModel, MessageToast, BaseController, Formatter) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.Launchpad", {

    formatter: Formatter,

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      const oCredentialsModel = new JSONModel({ username: "", password: "" });
      this.setViewModel(oCredentialsModel, "credentials");
      this._getIdentity();
    },

    onNavToHeats: function () {
      this.getRouter().navTo("heats", {}, false /* history */);
    },

    onNavToScoring: function () {
      this.getRouter().navTo("scoring", {}, false /* history */);
    },

    onNavToRaces: function () {
      this.getRouter().navTo("races", {}, false /* history */);
    },

    onNavToStatistics: function () {
      this.getRouter().navTo("statistics", {}, false /* history */);
    },

    onNavToKiosk: function () {
      this.getRouter().navTo("kiosk", {}, false /* history */);
    },

    onUserSubmit: function (oEvent) {
      this.byId("password").focus();
    },

    onPasswordSubmit: function (oEvent) {
      this.byId("login").focus();
      // perform login if return is pressed in password input field
      this.onLoginPress(oEvent);
    },

    onLoginPress: function (oEvent) {
      // close login popover when login button is pressed
      if (this._oPopover) {
        this._oPopover.close();
        delete this._oPopover;
      }
      this._login();
    },

    onShowLoginPress: function (oEvent) {
      const oControl = oEvent.getSource();

      if (!this._isAuthenticated()) {
        if (this._oPopover && this._oPopover.isOpen()) {
          // close login dialog if it's already open
          this._oPopover.close();
          delete this._oPopover;
        } else {
          // check if fragment is already loaded or not
          if (!this._pPopover) {
            // load fragment ...
            this._pPopover = Fragment.load({
              id: this.getView().getId(), name: "de.regatta_hd.infopoint.view.LoginPopover", controller: this
            }).then(function (oPopover) {
              // ... and initialize
              this.getView().addDependent(oPopover);
              oPopover.addStyleClass(this.getOwnerComponent().getContentDensityClass());
              return oPopover;
            }.bind(this));
          }

          // finish loading of fragment and open it
          this._pPopover.then(function (oPopover) {
            this._oPopover = oPopover;
            oPopover.openBy(oControl);
          }.bind(this));
        }
      } else {
        this._logout();
      }
    },

    _login: function () {
      const oCredentialsModel = this.getViewModel("credentials");
      const oCredentials = oCredentialsModel.getData();

      // see: https://api.jquery.com/jquery.ajax/
      $.ajax({
        type: "POST",
        data: JSON.stringify(oCredentials),
        url: "/api/login",
        contentType: "application/json",
        success: function (sResult) {
          this._updateUserModel(true, oCredentials.username);
          MessageToast.show(this.i18n("msg.loginSucceeded", undefined));
          $(".sapMMessageToast").removeClass("sapMMessageToastDanger").addClass("sapMMessageToastSuccess");
        }.bind(this),
        error: function (sResult) {
          MessageToast.show(this.i18n("msg.loginFailed", undefined));
          $(".sapMMessageToast").removeClass("sapMMessageToastSuccess").addClass("sapMMessageToastDanger");
        }.bind(this)
      });

      // reset password
      oCredentialsModel.setProperty("/password", "");
    },

    _logout: function () {
      $.ajax({
        type: "POST",
        url: "/api/logout",
        success: function (sResult) {
          this._updateUserModel(false, "");
        }.bind(this)
      });
    },

    _getIdentity: function () {
      $.ajax({
        type: "GET",
        url: "/api/identity",
        success: function (sResult) {
          this._updateUserModel(true, sResult);
        }.bind(this)
      });
    },

    _updateUserModel: function (bAuthenticated, sName) {
      const oUserModel = this.getViewModel("user");
      oUserModel.setProperty("/authenticated", bAuthenticated);
      oUserModel.setProperty("/name", sName);
    },

    _isAuthenticated: function () {
      return this.getViewModel("user").getProperty("/authenticated");
    }

  });
});