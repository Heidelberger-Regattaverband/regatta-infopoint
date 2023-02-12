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
      const sKioskUrl = this._getKioskUrl();
      if (!this._oKioskModel) {
        this._oKioskModel = await this.getJSONModel(sKioskUrl, this.getView());
        this.getOwnerComponent().setModel(this._oKioskModel, "kiosk");
      } else {
        await this.updateJSONModel(this._oKioskModel, sKioskUrl, this.getView());
      }

      const oData = this._oKioskModel.getData();
      if (!this._oFinishedModel) {
        this._oFinishedModel = await this.getJSONModel(this._getRegistrationsUrl(oData.finished[0].id), this.getView());
        this.getOwnerComponent().setModel(this._oFinishedModel, "finished");
      } else {
        await this.updateJSONModel(this._oFinishedModel, this._getRegistrationsUrl(oData.finished[0].id), this.getView());
      }

      if (!this._oNextModel) {
        this._oNextModel = await this.getJSONModel(this._getRegistrationsUrl(oData.next[0].id), this.getView());
        this.getOwnerComponent().setModel(this._oNextModel, "next");
      } else {
        await this.updateJSONModel(this._oNextModel, this._getRegistrationsUrl(oData.next[0].id), this.getView());
      }
    },

    _getKioskUrl: function () {
      return "/api/regattas/" + this.getRegattaId() + "/kiosk";
    },

    _getRegistrationsUrl: function (sHeatId) {
      return "/api/heats/" + sHeatId + "/registrations";
    }

  });
});