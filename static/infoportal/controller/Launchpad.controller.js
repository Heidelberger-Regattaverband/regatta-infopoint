sap.ui.define([
  'sap/ui/core/Fragment',
  "sap/ui/model/json/JSONModel",
  "de/regatta_hd/infopoint/controller/Base.controller",
  "../model/Formatter"
], function (Fragment, JSONModel, BaseController, Formatter) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.Launchpad", {

    formatter: Formatter,

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());
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

    showLoginButtonPress: function (oEvent) {
      const oControl = oEvent.getSource();

      if (this._oPopover) {
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

            const oCredentialsModel = new JSONModel({ user: "", password: "" });
            this.getView().setModel(oCredentialsModel, "credentials");

            return oPopover;
          }.bind(this));
        }

        // finish loading of fragment and open it
        this._pPopover.then(function (oPopover) {
          this._oPopover = oPopover;
          oPopover.openBy(oControl);
        }.bind(this));
      }
    },

    performLoginButtonPress: function (oEvent) {
      if (this._oPopover) {
        this._oPopover.close();
        delete this._oPopover;
      }
      const oCredentialsModel = this.getView().getModel("credentials");

      // TODO: perform login

      // reset password
      oCredentialsModel.setProperty("/password", "");
    }

  });
});