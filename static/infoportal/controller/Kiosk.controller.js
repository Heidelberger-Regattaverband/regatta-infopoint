sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller",
  "sap/ui/model/json/JSONModel",
  "../model/Formatter"
], function (BaseController, JSONModel, Formatter) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.Statistics", {

    formatter: Formatter,

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this._iIndexFinished = 0;
      this._iIndexNext = 0;

      this._oHeatFinishedModel = new JSONModel();
      this.getView().setModel(this._oHeatFinishedModel, "heatFinished");
      this._oHeatNextModel = new JSONModel();
      this.getView().setModel(this._oHeatNextModel, "heatNext");

      this.getRouter().getRoute("kiosk").attachMatched(_ => {
        this._loadKioskModel();
        this._iIntervalId = setInterval(this._updateModels.bind(this), 10000);
      }, this);

    },

    onNavBack: function () {
      if (this._iIntervalId) {
        clearInterval(this._iIntervalId);
        delete (this._iIntervalId);
      }
      this.navBack("startpage");
    },

    onRefreshButtonPress: function (oEvent) {
      this._loadKioskModel();
    },

    _updateModels: async function () {
      this._iIndexFinished += 1;
      this._iIndexNext += 1;
      if (this._iIndexFinished >= this._oKioskModel.getData().finished.length) {
        this._iIndexFinished = 0;
      }
      if (this._iIndexNext >= this._oKioskModel.getData().next.length) {
        this._iIndexNext = 0;
      }

      const oData = this._oKioskModel.getData();
      this._oHeatFinishedModel.setData(oData.finished[this._iIndexFinished]);
      this._oHeatNextModel.setData(oData.next[this._iIndexNext]);

      await Promise.all([this._loadRegsFinishedModel(oData.finished[this._iIndexFinished].id), this._loadRegsNextModel(oData.next[this._iIndexNext].id)]);
    },

    _loadRegsFinishedModel: async function (iHeatId) {
      if (!this._oFinishedModel) {
        this._oFinishedModel = await this.getJSONModel(this._getRegistrationsUrl(iHeatId), this.getView());
        this.getView().setModel(this._oFinishedModel, "regsFinished");
      } else {
        await this.updateJSONModel(this._oFinishedModel, this._getRegistrationsUrl(iHeatId), this.getView());
      }
    },

    _loadRegsNextModel: async function (iHeatId) {
      if (!this._oNextModel) {
        this._oNextModel = await this.getJSONModel(this._getRegistrationsUrl(iHeatId), this.getView());
        this.getView().setModel(this._oNextModel, "regsNext");
      } else {
        await this.updateJSONModel(this._oNextModel, this._getRegistrationsUrl(iHeatId), this.getView());
      }
    },

    _loadKioskModel: async function () {
      if (!this._oKioskModel) {
        this._oKioskModel = await this.getJSONModel(this._getKioskUrl(), this.getView());
        this.getView().setModel(this._oKioskModel, "kiosk");
      } else {
        await this.updateJSONModel(this._oKioskModel, this._getKioskUrl(), this.getView());
      }

      this._updateModels();
    },

    _getKioskUrl: function () {
      return "/api/regattas/" + this.getRegattaId() + "/kiosk";
    },

    _getRegistrationsUrl: function (sHeatId) {
      return "/api/heats/" + sHeatId + "/registrations";
    }

  });
});