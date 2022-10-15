sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller",
  "sap/ui/model/json/JSONModel",
  "../model/Formatter"
], function (BaseController, JSONModel, Formatter) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.HeatRegistrationsTable", {

    formatter: Formatter,

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());
      this.getRouter().getRoute("heatRegistrations").attachMatched(this._loadModel, this);
    },

    onNavBack: function () {
      this.navBack("heats");
    },

    handlePrevious: function () {
      this.getEventBus().publish("heat", "previous", {});
      this._loadModel();
    },

    handleNext: function () {
      this.getEventBus().publish("heat", "next", {});
      this._loadModel();
    },

    _loadModel: function () {
      const oHeat = this.getOwnerComponent().getModel("heat").getData();
      const oModel = new JSONModel();
      oModel.loadData("/api/heats/" + oHeat.id + "/registrations");
      this.getOwnerComponent().setModel(oModel, "heatRegistrations");
    }

  });
});