sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller",
  "sap/ui/model/json/JSONModel",
  "../model/Formatter"
], function (BaseController, JSONModel, Formatter) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.Heats", {

    formatter: Formatter,

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this.getRouter().getRoute("heats").attachMatched(this._loadHeatsModel, this);
    },

    onNavBack: function () {
      this.navBack("startpage");
    },

    _loadHeatsModel: function () {
      const oRegatta = this.getOwnerComponent().getModel("regatta").getData();
      const oHeatsModel = new JSONModel();
      oHeatsModel.loadData("/api/regattas/" + oRegatta.id + "/heats");
      this.getOwnerComponent().setModel(oHeatsModel, "heats");
    }

  });
});