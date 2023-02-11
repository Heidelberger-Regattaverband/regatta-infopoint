sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller",
  "../model/Formatter"
], function (BaseController, Formatter) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.Statistics", {

    formatter: Formatter,

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this.getRouter().getRoute("kiosk").attachMatched(this._loadKioskModel, this);
    },

    onNavBack: function () {
      this.navBack("startpage");
    },

    onRefreshButtonPress: function (oEvent) {
      this._loadKioskModel();
    },

    _loadKioskModel: async function () {
      if (!this._oKioskModel) {
        this._oKioskModel = await this.getJSONModel("/api/regattas/" + this.getRegattaId() + "/heats_kiosk", this.getView());
        this.getOwnerComponent().setModel(this._oKioskModel, "heats");
      } else {
        await this.updateJSONModel(this._oKioskModel, "/api/regattas/" + this.getRegattaId() + "/heats_kiosk", this.getView());
      }

      const oData = this._oKioskModel.getData();

      if (!this._oFinishedModel) {
        this._oFinishedModel = await this.getJSONModel("/api/heats/" + oData[0].id + "/registrations", this.getView());
        this.getOwnerComponent().setModel(this._oFinishedModel, "finished");
      } else {
        await this.updateJSONModel(this._oFinishedModel, "/api/heats/" + oData[0].id + "/registrations", this.getView());
      }

      if (!this._oNextModel) {
        this._oNextModel = await this.getJSONModel("/api/heats/" + oData[1].id + "/registrations", this.getView());
        this.getOwnerComponent().setModel(this._oNextModel, "next");
      } else {
        await this.updateJSONModel(this._oNextModel, "/api/heats/" + oData[1].id + "/registrations", this.getView());
      }
    }

  });
});